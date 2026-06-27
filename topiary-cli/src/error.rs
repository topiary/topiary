use rootcause::{
    Report,
    markers::{Dynamic, Mutable, SendSync},
    report,
};
use rootcause_preformat::{PreformatReportExt, PreformattedContext};
use std::{any::TypeId, error, fmt, io, process::ExitCode, result};
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
            TopiaryError::Config => write!(f, "Configuration error"),
            TopiaryError::Multiple => write!(
                f,
                "Processing of one or more inputs failed; see below for details"
            ),
            TopiaryError::UnsupportedLanguage(name) => {
                write!(f, "The specified language is unsupported: {name}")
            }
            TopiaryError::CheckFailed {
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
        }
    }
}

// source is handled by `rootcause::Report::current_context_error_source`
impl error::Error for TopiaryError {}

pub(crate) fn exit_code<C>(r: Report<C>) -> ExitCode
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
        }
        // NOTE/TODO: this does not currently handle type erased variants of original types
        // see
        // https://docs.rs/rootcause-preformat/latest/rootcause_preformat/struct.PreformattedContext.html#method.original_type_id
        // for more
        if let Some(e) = rep.downcast_current_context::<TopiaryConfigError>() {
            // I/O errors: Exit 3
            code = match e {
                TopiaryConfigError::FileNotFound(_)
                | TopiaryConfigError::QueryFileNotFound(_)
                | TopiaryConfigError::Io(_)
                | TopiaryConfigError::Fetching(
                    FetchError::Io(_) | FetchError::GrammarFileNotFound(_),
                ) => 3,
                _ => 10,
            };
            break;
        }
        if let Some(e) = rep.downcast_current_context::<TopiaryError>() {
            code = match e {
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
        iter_downcast_reports::<FormatterError>(self)
            .any(|fmt_err| *fmt_err == FormatterError::PatternDoesNotMatch)
    }
}
pub(crate) trait ResultPreformat<T, C> {
    fn preformat_context(self) -> Result<T, Report<PreformattedContext>>;
}

impl<T, C: 'static> ResultPreformat<T, C> for Result<T, C> {
    fn preformat_context(self) -> Result<T, Report<PreformattedContext>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(report!(e).preformat()),
        }
    }
}

fn iter_downcast_reports<T: 'static>(report: &Report<impl ?Sized>) -> impl Iterator<Item = &T> {
    report
        .iter_reports()
        .filter_map(|r| r.downcast_current_context::<T>())
}
