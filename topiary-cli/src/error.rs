use std::{error, fmt, io, path::PathBuf, process::ExitCode, result};

use similar::TextDiff;
use topiary_config::error::{TopiaryConfigError, TopiaryConfigFetchingError};
use topiary_core::FormatterError;

/// A convenience wrapper around `std::result::Result<T, TopiaryError>`.
pub type CLIResult<T> = result::Result<T, TopiaryError>;

/// The errors that can be raised by either the Topiary CLI, or passed through by the formatter
/// library code. This acts as a supertype of `FormatterError`, with additional members to denote
/// CLI-specific failures.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum TopiaryError {
    Lib(FormatterError),
    Bin(String, Option<CLIError>),
    Config(topiary_config::error::TopiaryConfigError),
}

/// A subtype of `TopiaryError::Bin`
#[derive(Debug)]
pub enum CLIError {
    IOError(io::Error),
    Generic(Box<dyn error::Error>),
    Multiple,
    UnsupportedLanguage(String),

    /// Formatting check failed: input is not already formatted
    CheckFailed {
        source_name: String,
        original: String,
        formatted: String,
    },

    /// Could not detect the input language from the `(filename, Option<extension>)`
    LanguageDetection(PathBuf, Option<String>),
}

/// # Safety
///
/// Something can safely be Send unless it shares mutable state with something
/// else without enforcing exclusive access to it. TopiaryError does not have a
/// mutable state.
unsafe impl Send for TopiaryError {}

/// # Safety
///
/// Something can safely be Sync if and only if no other &TopiaryError can write
/// to it. Since our TopiaryError contains no mutable data, TopiaryError is Sync.
unsafe impl Sync for TopiaryError {}

impl fmt::Display for TopiaryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TopiaryError::Lib(error) => write!(f, "{error}"),
            TopiaryError::Bin(
                _,
                Some(CLIError::CheckFailed {
                    source_name,
                    original,
                    formatted,
                }),
            ) => {
                let diff = TextDiff::from_lines(original, formatted);
                write!(
                    f,
                    "Diff in {source_name}:\n{}",
                    diff.unified_diff()
                        .context_radius(3)
                        .header("original", "formatted")
                )
            }
            TopiaryError::Bin(message, _) => write!(f, "{message}"),
            TopiaryError::Config(e) => write!(f, "{e}"),
        }
    }
}

impl error::Error for TopiaryError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            TopiaryError::Lib(error) => error.source(),
            TopiaryError::Bin(_, Some(CLIError::IOError(error))) => Some(error),
            TopiaryError::Bin(_, Some(CLIError::Generic(error))) => error.source(),
            TopiaryError::Bin(_, Some(CLIError::Multiple)) => None,
            TopiaryError::Bin(_, Some(CLIError::UnsupportedLanguage(_))) => None,
            TopiaryError::Bin(_, Some(CLIError::CheckFailed { .. })) => None,
            TopiaryError::Bin(_, Some(CLIError::LanguageDetection(_, _))) => None,
            TopiaryError::Bin(_, None) => None,
            TopiaryError::Config(error) => error.source(),
        }
    }
}

impl From<TopiaryError> for ExitCode {
    fn from(e: TopiaryError) -> Self {
        let exit_code = match e {
            // Things went well but Topiary needs to answer 'false' in a clean way: Exit 1
            _ if e.benign() => 1,

            // Check mode detected unformatted files: Exit 1
            // This error is not benign, but we still need to answer `false` without resulting in a typical an error
            TopiaryError::Bin(_, Some(CLIError::CheckFailed { .. })) => 1,

            // Multiple errors: Exit 9
            TopiaryError::Bin(_, Some(CLIError::Multiple)) => 9,

            // Idempotency parsing errors: Exit 8
            TopiaryError::Lib(FormatterError::IdempotenceParsing(_)) => 8,

            // Idempotency errors: Exit 7
            TopiaryError::Lib(FormatterError::Idempotence) => 7,

            // Exit 6 no longer exists and is now reserved for compatibility reasons

            // Parsing errors: Exit 5
            TopiaryError::Lib(FormatterError::Parsing { .. }) => 5,

            // Query errors: Exit 4
            TopiaryError::Lib(FormatterError::Query(_, _)) => 4,

            // I/O errors: Exit 3
            TopiaryError::Lib(FormatterError::Io(_))
            | TopiaryError::Bin(_, Some(CLIError::IOError(_))) => 3,

            // Bad arguments: Exit 2
            // (Handled by clap: https://github.com/clap-rs/clap/issues/3426)

            // Anything else: Exit 10
            _ => 10,
        };

        ExitCode::from(exit_code)
    }
}

impl From<FormatterError> for TopiaryError {
    fn from(e: FormatterError) -> Self {
        Self::Lib(e)
    }
}

impl From<TopiaryConfigError> for TopiaryError {
    fn from(e: TopiaryConfigError) -> Self {
        Self::Config(e)
    }
}

impl From<TopiaryConfigFetchingError> for TopiaryError {
    fn from(e: TopiaryConfigFetchingError) -> Self {
        Self::Config(TopiaryConfigError::Fetching(e))
    }
}

impl From<io::Error> for TopiaryError {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::NotFound => {
                Self::Bin("File not found".into(), Some(CLIError::IOError(e)))
            }

            _ => Self::Bin(
                "Could not read or write to file".into(),
                Some(CLIError::IOError(e)),
            ),
        }
    }
}

impl From<tempfile::PersistError> for TopiaryError {
    fn from(e: tempfile::PersistError) -> Self {
        Self::Bin(
            "Could not persist output to disk".into(),
            Some(CLIError::IOError(e.error)),
        )
    }
}

// We only have to deal with io::BufWriter<crate::output::OutputFile>,
// but the genericised code is clearer
impl<W> From<io::IntoInnerError<W>> for TopiaryError
where
    W: io::Write + fmt::Debug + Send + 'static,
{
    fn from(e: io::IntoInnerError<W>) -> Self {
        Self::Bin(
            "Could not flush internal buffer".into(),
            Some(CLIError::Generic(Box::new(e))),
        )
    }
}

impl From<tokio::task::JoinError> for TopiaryError {
    fn from(e: tokio::task::JoinError) -> Self {
        TopiaryError::Bin(
            "Could not join parallel formatting tasks".into(),
            Some(CLIError::Generic(Box::new(e))),
        )
    }
}

// Tells whether an error should raise a message on stderr,
// or if it's an "expected" error.
pub trait Benign {
    fn benign(&self) -> bool;
}

impl Benign for TopiaryError {
    #[allow(clippy::match_like_matches_macro)]
    fn benign(&self) -> bool {
        match self {
            TopiaryError::Lib(FormatterError::PatternDoesNotMatch) => true,
            _ => false,
        }
    }
}

pub(crate) fn print_error(e: &dyn error::Error) {
    log::error!("{e}");
    if let Some(source) = e.source() {
        log::error!("Cause: {source}");
    }
}
