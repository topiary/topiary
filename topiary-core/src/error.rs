//! This module defines all errors that might be propagated out of the library,
//! including all of the trait implementations one might expect for Errors.

use std::{any::TypeId, error::Error, fmt, io, ops::Deref, path::PathBuf, str, string};

use itertools::Itertools;
use miette::{Diagnostic, NamedSource, SourceSpan};
use rootcause::{
    Report, ReportConversion,
    handlers::Any,
    markers::{self, Local, SendSync},
    prelude::*,
    report_attachments::ReportAttachments,
};
use topiary_tree_sitter_facade::{Point, QueryError, Range};

use crate::tree_sitter::NodeSpan;

/// The various errors the formatter may return.
#[derive(Debug)]
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

    // Tree-sitter could not parse the input without errors.
    Parsing,

    /// The query contains a pattern that had no match in the input file.
    PatternDoesNotMatch,

    /// There was an error in the query file. If this happened using our
    /// provided query files, it is a bug. Please log an issue.
    Query(String),

    /// I/O-related errors
    Io(String),
}

// impl FormatterError {
//     fn get_span(&mut self) -> Option<&mut NodeSpan> {
//         match self {
//             Self::Parsing(span) => Some(span),
//             Self::IdempotenceParsing(err) => err.get_span(),
//             _ => None,
//         }
//     }
//     pub fn with_content(mut self, content: String) -> Self {
//         if let Some(span) = self.get_span() {
//             span.set_content(content);
//         }
//         self
//     }
//
//     pub fn with_location(mut self, location: String) -> Self {
//         if let Some(span) = self.get_span() {
//             span.set_location(location);
//         }
//         self
//     }
// }

// pub trait GetSpan {
//     fn get_or_init(&mut self) -> ErrorSpan;
// }
//
// impl GetSpan for Report<FormatterError> {
//     fn get_or_init(&mut self) -> ErrorSpan {
//         let attachments = self.attachments_mut();
//         let new_attachments = ReportAttachments
//         while let Some(a) = attachments.pop() {
//         }
//         let span_idx = attachments
//             .iter()
//             .find_position(|a| a.inner_type_id() == TypeId::of::<ErrorSpan>())
//             .map(|(idx, a)| idx);
//         if let Some(idx) = span_idx {
//             attachments.pop()
//         }
//     }
// }

impl fmt::Display for FormatterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let please_log_message = "If this happened with the built-in query files, it is a bug. It would be\nhelpful if you logged this error at\nhttps://github.com/tweag/topiary/issues/new?assignees=&labels=type%3A+bug&template=bug_report.md";
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

                // let report = miette::Report::new(ErrorSpan::from(span));
                // write!(f, "{report:?}")
            }

            Self::PatternDoesNotMatch => {
                write!(
                    f,
                    "The query contains a pattern that does not match the input"
                )
            }

            Self::Internal(message) | Self::Query(message) | Self::Io(message) => {
                write!(f, "{message}")
            }
        }
    }
}

impl Error for FormatterError {}

macro_rules! report_conversion {
    ($from:path, $context:expr) => {
        impl<T> ReportConversion<$from, markers::Mutable, T> for FormatterError
        where
            Self: markers::ObjectMarkerFor<T>,
        {
            fn convert_report(
                report: Report<$from, markers::Mutable, T>,
            ) -> Report<Self, markers::Mutable, T> {
                report.context($context)
            }
        }
    };
}

report_conversion!(
    std::str::Utf8Error,
    FormatterError::Io("Input is not valid UTF-8".to_string())
);

report_conversion!(
    std::string::FromUtf8Error,
    FormatterError::Io("Input is not valid UTF-8".to_string())
);

report_conversion!(
    std::fmt::Error,
    FormatterError::Io("Failed to format output".to_string())
);

report_conversion!(
    serde_json::Error,
    FormatterError::Io("Could not serialise JSON output".to_string())
);

report_conversion!(
    topiary_tree_sitter_facade::LanguageError,
    FormatterError::Io("Error while loading language grammar".to_string())
);

report_conversion!(
    topiary_tree_sitter_facade::ParserError,
    FormatterError::Io("Error while parsing".to_string())
);

report_conversion!(
    topiary_tree_sitter_facade::QueryError,
    FormatterError::Query("Error parsing query file".to_string())
);

// We only have to deal with io::BufWriter<Vec<u8>>, but the genericised code is
// clearer
impl<W, T> ReportConversion<io::IntoInnerError<W>, markers::Mutable, T> for FormatterError
where
    Self: markers::ObjectMarkerFor<T>,
    W: io::Write + fmt::Debug + Send + 'static,
{
    fn convert_report(
        report: Report<io::IntoInnerError<W>, markers::Mutable, T>,
    ) -> Report<Self, markers::Mutable, T> {
        report.context(Self::Io("Cannot flush internal buffer".to_string()))
    }
}

impl<T> ReportConversion<io::Error, markers::Mutable, T> for FormatterError
where
    Self: markers::ObjectMarkerFor<T>,
{
    fn convert_report(
        report: Report<io::Error, markers::Mutable, T>,
    ) -> Report<Self, markers::Mutable, T> {
        let msg = match report.current_context().kind() {
            io::ErrorKind::NotFound => "File not found",
            _ => "Could not read or write to file",
        };

        report.context(Self::Io(msg.to_string()))
    }
}

pub struct Filename(pub PathBuf);
pub struct Source(pub String);
pub struct Language(pub &'static str);

// data structure used to illustrate code that is being formatted
// or used as a query
#[derive(Diagnostic, Debug)]
pub(crate) struct ErrorSpan {
    #[source_code]
    src: NamedSource<String>,
    // TODO handle different labeling for `QueryError`s
    #[label("(ERROR) node")]
    span: SourceSpan,
    range: Range,
}

impl std::fmt::Display for ErrorSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = self.range.start_point();
        let end = self.range.end_point();
        write!(
            f,
            "Parsing error between line {}, column {} and line {}, column {}",
            start.row(),
            start.column(),
            end.row(),
            end.column()
        )
    }
}

impl std::error::Error for ErrorSpan {}

impl From<NodeSpan> for ErrorSpan {
    fn from(span: NodeSpan) -> Self {
        Self {
            src: NamedSource::new(
                span.location.clone().unwrap_or_default(),
                span.content.clone().unwrap_or_default(),
            )
            .with_language(span.language),
            span: span.source_span(),
            range: span.range,
        }
    }
}
