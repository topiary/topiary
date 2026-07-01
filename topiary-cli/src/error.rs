use rootcause::{
    Report, ReportRef,
    markers::{Dynamic, Mutable, SendSync, Uncloneable},
    report,
    report_collection::ReportCollection,
};
use rootcause_preformat::PreformatReportExt;
use std::{error, fmt, io, process::ExitCode, result};
use topiary_config::error::{TopiaryConfigError, TopiaryConfigFetchingError as FetchError};

use similar::TextDiff;
use topiary_core::FormatterError;

/// A convenience wrapper around `std::result::Result<T, TopiaryError>`.
pub type CLIResult<C, T = SendSync> = result::Result<C, Report<Dynamic, Mutable, T>>;

/// The errors that can be raised by either the Topiary CLI, or passed through by the formatter
/// library code. This acts as a supertype of `FormatterError`, with additional members to denote
/// CLI-specific failures.
#[derive(Debug)]
pub enum TopiaryError {
    Io,
    Config,
    Multiple,
    UnsupportedLanguage(String),
    /// Formatting check failed: input is not already formatted
    CheckFailed {
        source_name: String,
        original: String,
        formatted: String,
    },
}

impl fmt::Display for TopiaryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Config => write!(f, "Configuration error"),
            Self::Multiple => write!(
                f,
                "Processing of one or more inputs failed; see below for details"
            ),
            Self::UnsupportedLanguage(name) => {
                write!(f, "The specified language is unsupported: {name}")
            }
            Self::CheckFailed {
                source_name,
                original,
                formatted,
            } => {
                writeln!(f, "{source_name} is not formatted")?;
                let diff = TextDiff::from_lines(original, formatted);
                write!(
                    f,
                    "Diff in {source_name}:\n{}",
                    diff.unified_diff()
                        .context_radius(3)
                        .header("original", "formatted")
                )
            }
            Self::Io => {
                write!(f, "I/O Error")
            }
        }
    }
}

// source is handled by `rootcause::Report::current_context_error_source`
impl error::Error for TopiaryError {}

pub(crate) fn exit_code<C>(r: &Report<C>) -> ExitCode
where
    C: ?Sized,
{
    // Things went well but Topiary needs to answer 'false' in a clean way: Exit 1
    if r.benign() {
        return ExitCode::FAILURE;
    }

    // Anything not explicitly covered returns an ExitCode of 10
    // Bad arguments: Exit 2
    // (Handled by clap: https://github.com/clap-rs/clap/issues/3426)
    let mut code = 10;
    for rep in r.iter_reports() {
        if let Some(e) = rep.downcast_current_context::<FormatterError>() {
            code = match e {
                // TODO check failed
                // I/O errors: Exit 3
                FormatterError::Io => 3,
                // Query errors: Exit 4
                FormatterError::Query(_) => 4,
                // Parsing errors: Exit 5
                FormatterError::Parsing => 5,
                // Idempotency errors: Exit 7
                FormatterError::Idempotence => 7,
                // Idempotency parsing errors: Exit 8
                FormatterError::IdempotenceParsing => 8,
                _ => 10,
            };
            break;
        }
        if rep.downcast_current_context::<io::Error>().is_some() {
            // I/O errors: Exit 3
            code = 3;
            break;
        }

        if let Some(e) = rep.downcast_current_context::<TopiaryError>() {
            code = match e {
                // Check mode detected unformatted files: Exit 1
                // This error is not benign, but we still need to answer `false` without resulting in a typical an error
                TopiaryError::CheckFailed { .. } => 1,
                // I/O errors: Exit 3
                TopiaryError::Io => 3,
                // Multiple errors: Exit 9
                TopiaryError::Multiple => 9,
                // Anything else: Exit 10
                _ => 10,
            };
            break;
        }
    }

    ExitCode::from(code)
}

// Tells whether an error should raise a message on stderr,
// or if it's an "expected" error.
pub trait Benign {
    fn benign(&self) -> bool;
}

impl<C> Benign for Report<C>
where
    C: ?Sized,
{
    fn benign(&self) -> bool {
        let serious_err_in_collections =
            iter_downcast_reports::<ReportCollection, _>(self.as_ref())
                .flat_map(|c| c.iter())
                .any(|r| {
                    r.downcast_current_context::<FormatterError>()
                        != Some(&FormatterError::PatternDoesNotMatch)
                });
        if serious_err_in_collections {
            return false;
        }

        iter_downcast_reports::<FormatterError, _>(self.as_ref())
            .any(|fmt_err| *fmt_err == FormatterError::PatternDoesNotMatch)
    }
}

impl From<&TopiaryConfigError> for TopiaryError {
    fn from(value: &TopiaryConfigError) -> Self {
        match value {
            TopiaryConfigError::FileNotFound(_)
            | TopiaryConfigError::QueryFileNotFound(_)
            | TopiaryConfigError::Io(_)
            | TopiaryConfigError::Fetching(
                FetchError::Io(_) | FetchError::GrammarFileNotFound(_),
            ) => Self::Io,
            _ => Self::Config,
        }
    }
}
impl<T, O> From<&Report<TopiaryConfigError, O, T>> for TopiaryError {
    fn from(value: &Report<TopiaryConfigError, O, T>) -> Self {
        value.current_context().into()
    }
}

pub(crate) trait ResultPreformat<T, C> {
    fn preformat_context(self) -> Result<T, Report<TopiaryError>>;
}

impl<T, C: 'static> ResultPreformat<T, C> for Result<T, C>
where
    C: 'static,
    for<'a> TopiaryError: From<&'a C>,
{
    fn preformat_context(self) -> Result<T, Report<TopiaryError>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => {
                let cli_err = TopiaryError::from(&e);
                Err(report!(e).preformat().context(cli_err))
            }
        }
    }
}

fn iter_downcast_reports<T: 'static, C>(
    report: ReportRef<'_, C, Uncloneable>,
) -> impl Iterator<Item = &T>
where
    C: ?Sized,
{
    report
        .iter_reports()
        .filter_map(|r| r.downcast_current_context::<T>())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use rootcause::{handlers, markers::Cloneable, report_attachments::ReportAttachments};

    fn assert_exit_code<C: ?Sized>(r: Report<C>, expected: u8) {
        assert_eq!(exit_code(&r), ExitCode::from(expected));
    }

    #[test]
    fn preformat_context_io_variant_exits_3() {
        let err: Result<(), TopiaryConfigError> = Err(TopiaryConfigError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "missing",
        )));
        let report = err.preformat_context().unwrap_err();
        assert_exit_code(report, 3);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn preformat_context_fetching_io_exits_3() {
        let err: Result<(), TopiaryConfigError> = Err(TopiaryConfigError::Fetching(
            FetchError::Io(io::Error::new(io::ErrorKind::PermissionDenied, "denied")),
        ));
        let report = err.preformat_context().unwrap_err();
        assert_exit_code(report, 3);
    }

    #[test]
    fn preformat_context_unknown_language_exits_10() {
        let err: Result<(), TopiaryConfigError> =
            Err(TopiaryConfigError::UnknownLanguage("nope".to_string()));
        let report = err.preformat_context().unwrap_err();
        assert_exit_code(report, 10);
    }

    #[test]
    fn preformat_context_ok_passthrough() {
        let ok: Result<u32, TopiaryConfigError> = Ok(42);
        assert_eq!(ok.preformat_context().unwrap(), 42);
    }

    #[test]
    fn iter_downcast_exit_code() {
        let mut collection = report!(ReportCollection::from_iter(vec![
            report!(FormatterError::PatternDoesNotMatch).into_dynamic(),
            // preformatted io error, should exit 3
            Err::<(), _>(TopiaryConfigError::FileNotFound(PathBuf::new()))
                .preformat_context()
                .unwrap_err()
                .into_dynamic(),
        ]));

        assert_eq!(exit_code(&collection), 3.into());

        // remote IO error
        collection.current_context_mut().pop();
        assert_eq!(exit_code(&collection), 1.into());
    }
}
