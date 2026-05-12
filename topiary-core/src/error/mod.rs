//! This module defines all errors that might be propagated out of the library,
//! including all of the trait implementations one might expect for Errors.

use std::{error::Error, fmt, io};

use rootcause::{Report, ReportConversion, markers};

pub use error_span::{ErrorSpan, SpanAttachment};

mod error_span;

/// The various errors the formatter may return.
#[derive(Debug, PartialEq)]
pub enum FormatterError {
    /// The input produced output that isn't idempotent, i.e. formatting the
    /// output again made further changes. If this happened using our provided
    /// query files, it is a bug. Please log an issue.
    Idempotence,

    /// The input produced invalid output, i.e. formatting the output again led
    /// to a parsing error. If this happened using our provided query files, it
    /// is a bug. Please log an issue.
    IdempotenceParsing,

    /// An internal error occurred. This is a bug. Please log an issue.
    Internal(String),

    /// An injected language could not be resolved.
    InjectionLanguageResolution { language: String },

    // Tree-sitter could not parse the input without errors.
    Parsing,

    /// The query contains a pattern that had no match in the input file.
    PatternDoesNotMatch,

    /// There was an error in the query file. If this happened using our
    /// provided query files, it is a bug. Please log an issue.
    Query(String),

    /// I/O-related errors
    Io,
}

impl fmt::Display for FormatterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let please_log_message = "If this happened with the built-in query files, it is a bug. It would be\nhelpful if you logged this error at\nhttps://github.com/topiary/topiary/issues/new?assignees=&labels=type%3A+bug&template=bug_report.md";
        match self {
            Self::Idempotence => {
                write!(
                    f,
                    "The formatter did not produce the same\nresult when invoked twice (idempotence check).\n\n{please_log_message}"
                )
            }

            Self::IdempotenceParsing => {
                write!(
                    f,
                    "The formatter produced invalid output and\nfailed when trying to format twice (idempotence check).\n\n{please_log_message}\n\nThe following is the error received when running the second time, but note\nthat any line and column numbers refer to the formatted code, not the\noriginal input. Run Topiary with the --skip-idempotence flag to see this\ninvalid formatted code."
                )
            }

            Self::Parsing => {
                write!(f, "Tree-sitter could not parse the input without errors.")
            }

            Self::PatternDoesNotMatch => {
                write!(
                    f,
                    "The query contains a pattern that does not match the input"
                )
            }
            Self::Io => {
                write!(f, "I/O Error")
            }
            Self::Internal(message) | Self::Query(message) => {
                write!(f, "{message}")
            }

            Self::InjectionLanguageResolution { language, .. } => {
                write!(f, "Could not resolve injected language \"{language}\"")
            }
        }
    }
}

impl Error for FormatterError {}

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
                    let report = report.context($err::$variant);
                    let report = report.attach($msg);
                    report

                }
            }
        )+
    };
}

report_conversion!(
    std::str::Utf8Error | std::string::FromUtf8Error,
    FormatterError::Io,
    "Input is not valid UTF-8"
);

report_conversion!(
    std::fmt::Error,
    FormatterError::Io,
    "Failed to format output"
);

report_conversion!(
    serde_json::Error,
    FormatterError::Io,
    "Could not serialise JSON output"
);

report_conversion!(
    topiary_tree_sitter_facade::LanguageError,
    FormatterError::Parsing,
    "Error while loading language grammar"
);

report_conversion!(
    topiary_tree_sitter_facade::ParserError,
    FormatterError::Parsing,
    "Error while parsing"
);

// We only have to deal with io::BufWriter<Vec<u8>>, but the genericised code is
// clearer
impl<W, T> ReportConversion<io::IntoInnerError<W>, markers::Mutable, T> for FormatterError
where
    Self: markers::ObjectMarkerFor<T>,
    W: io::Write + fmt::Debug + Send + 'static,
    &'static str: markers::ObjectMarkerFor<T>,
{
    fn convert_report(
        report: Report<io::IntoInnerError<W>, markers::Mutable, T>,
    ) -> Report<Self, markers::Mutable, T> {
        report
            .context(Self::Io)
            .attach("Cannot flush internal buffer")
    }
}

impl<T> ReportConversion<io::Error, markers::Mutable, T> for FormatterError
where
    Self: markers::ObjectMarkerFor<T>,
    io::ErrorKind: markers::ObjectMarkerFor<T>,
    &'static str: markers::ObjectMarkerFor<T>,
{
    fn convert_report(
        report: Report<io::Error, markers::Mutable, T>,
    ) -> Report<Self, markers::Mutable, T> {
        let kind = report.current_context().kind();
        let msg = match kind {
            io::ErrorKind::NotFound => "File not found",
            _ => "Could not read or write to file",
        };

        report.context(Self::Io).attach(msg).attach(kind)
    }
}
