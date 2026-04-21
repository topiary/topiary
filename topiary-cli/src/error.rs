use rootcause::{
    Report, ReportConversion,
    markers::{self, Local, Mutable, ObjectMarkerFor, SendSync},
    report,
};
use std::{error, fmt, io, process::ExitCode, result};
use topiary_config::error::{TopiaryConfigError, TopiaryConfigFetchingError};
use topiary_core::FormatterError;
use topiary_tree_sitter_facade::QueryError;

/// A convenience wrapper around `std::result::Result<T, TopiaryError>`.
pub type CLIResult<C, T = SendSync> = result::Result<C, Report<TopiaryError, Mutable, T>>;

/// The errors that can be raised by either the Topiary CLI, or passed through by the formatter
/// library code. This acts as a supertype of `FormatterError`, with additional members to denote
/// CLI-specific failures.
#[derive(Debug)]
pub enum TopiaryError {
    // formatter errors or general errors such as tree-sitter specific ones
    Lib,
    Config,
    /// I/O-related errors
    Io,
    Multiple,
    UnsupportedLanguage(String),
    Other,
}

impl fmt::Display for TopiaryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TopiaryError::Lib => write!(f, "Formatter error"),
            TopiaryError::Io => write!(f, "I/O Error"),
            TopiaryError::Config => write!(f, "Configuration error"),
            TopiaryError::Multiple => write!(
                f,
                "Processing of one or more inputs failed; see below for details"
            ),
            TopiaryError::UnsupportedLanguage(name) => {
                write!(f, "The specified language is unsupported: {name}")
            }
            TopiaryError::Other => todo!(),
        }
    }
}

// source is handled by `rootcause::Report::current_context_error_source`
impl error::Error for TopiaryError {}

pub(crate) fn exit_code<C>(r: Report<C, Mutable, Local>) -> ExitCode
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

        if let Some(e) = rep.downcast_current_context::<TopiaryError>() {
            code = match e {
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

impl<T> ReportConversion<tempfile::PersistError, Mutable, T> for TopiaryError
where
    Self: ObjectMarkerFor<T>,
    String: ObjectMarkerFor<T>,
{
    fn convert_report(
        report: Report<tempfile::PersistError, Mutable, T>,
    ) -> Report<Self, Mutable, T> {
        let filepath = format!("{}", report.current_context().file.path().display());
        report.context(TopiaryError::Io).attach(filepath)
    }
}

impl<T> ReportConversion<io::Error, Mutable, T> for TopiaryError
where
    Self: ObjectMarkerFor<T>,
    io::ErrorKind: ObjectMarkerFor<T>,
    &'static str: ObjectMarkerFor<T>,
{
    fn convert_report(report: Report<io::Error, Mutable, T>) -> Report<Self, Mutable, T> {
        let kind = report.current_context().kind();
        let msg = match kind {
            io::ErrorKind::NotFound => "File not found",
            _ => "Could not read or write to file",
        };

        report.context(Self::Io).attach(msg).attach(kind)
    }
}

// We only have to deal with io::BufWriter<crate::output::OutputFile>,
// but the genericised code is clearer
impl<W, T> ReportConversion<io::IntoInnerError<W>, Mutable, T> for TopiaryError
where
    Self: ObjectMarkerFor<T>,
    W: io::Write + fmt::Debug + Send + 'static,
    &'static str: ObjectMarkerFor<T>,
{
    fn convert_report(
        report: Report<io::IntoInnerError<W>, Mutable, T>,
    ) -> Report<Self, Mutable, T> {
        report
            .context(Self::Io)
            .attach("Cannot flush internal buffer")
    }
}

// Tells whether an error should raise a message on stderr,
// or if it's an "expected" error.
pub trait Benign {
    fn benign(&self) -> bool;
}

impl<C> Benign for Report<C, Mutable, Local>
where
    C: ?Sized,
{
    fn benign(&self) -> bool {
        if let Some(FormatterError::PatternDoesNotMatch) =
            iter_downcast_reports::<FormatterError>(self).next()
        {
            return true;
        }
        false
    }
}

// private convenience macro to do [`rootcause::ReportConversion`]
// https://docs.rs/rootcause/latest/rootcause/trait.ReportConversion.html
macro_rules! report_conversion {
    ($($from:ty)|+, $err:ident::$variant:ident, $msg:literal) => {
        $(
            impl<T> ReportConversion<$from, markers::Mutable, T> for $err
            where
                Self: markers::ObjectMarkerFor<T>,
                &'static str: markers::ObjectMarkerFor<T>,
            {
                fn convert_report(
                    report: Report<$from, markers::Mutable, T>,
                ) -> Report<Self, markers::Mutable, T> {
                    report.context($err::$variant).attach($msg)

                }
            }
        )+
    };

    ($($from:ty)|+, $err:ident::$variant:ident) => {
        $(
            impl<T> ReportConversion<$from, markers::Mutable, T> for $err
            where
                Self: markers::ObjectMarkerFor<T>,
            {
                fn convert_report(
                    report: Report<$from, markers::Mutable, T>,
                ) -> Report<Self, markers::Mutable, T> {
                    report.context($err::$variant)

                }
            }
        )+
    };
}

report_conversion!(
    tokio::task::JoinError,
    TopiaryError::Other,
    "Could not join parallel formatting tasks"
);

report_conversion!(FormatterError | QueryError, TopiaryError::Lib);

impl ReportConversion<TopiaryConfigError, Mutable, Local> for TopiaryError
where
    Self: ObjectMarkerFor<Local>,
    TopiaryConfigError: ObjectMarkerFor<Local>,
{
    fn convert_report(
        report: Report<TopiaryConfigError, Mutable, Local>,
    ) -> Report<Self, Mutable, Local> {
        report.context(TopiaryError::Config)
    }
}

impl ReportConversion<TopiaryConfigFetchingError, Mutable, Local> for TopiaryError
where
    Self: ObjectMarkerFor<Local>,
    TopiaryConfigFetchingError: ObjectMarkerFor<Local>,
{
    fn convert_report(
        report: Report<TopiaryConfigFetchingError, Mutable, Local>,
    ) -> Report<Self, Mutable, Local> {
        report.context(TopiaryError::Config)
    }
}

pub(crate) trait PreformatLocal<C> {
    fn preformat_context(self) -> Report<C>;
}

impl PreformatLocal<TopiaryError> for TopiaryConfigError {
    fn preformat_context(self) -> Report<TopiaryError> {
        report!(self).preformat().context(TopiaryError::Config)
    }
}

impl PreformatLocal<TopiaryError> for TopiaryConfigFetchingError {
    fn preformat_context(self) -> Report<TopiaryError> {
        report!(self).preformat().context(TopiaryError::Config)
    }
}

pub(crate) trait ResultPreformatLocal<T, C> {
    fn preformat_context(self) -> Result<T, Report<C>>;
}

impl<T, C, C2> ResultPreformatLocal<T, C2> for Result<T, C>
where
    C: PreformatLocal<C2>,
{
    fn preformat_context(self) -> Result<T, Report<C2>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.preformat_context()),
        }
    }
}

fn iter_downcast_reports<T: 'static>(
    report: &Report<impl ?Sized, Mutable, Local>,
) -> impl Iterator<Item = &T> {
    report
        .iter_reports()
        .filter_map(|r| r.downcast_current_context::<T>())
}
