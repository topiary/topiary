// The ErrorSpan struct uses Miette's Diagnostic derive macro, which reads fields through
// procedural macro attributes (#[source_code], #[label]). Newer versions of Clippy flag these
// field assignments as unused, even though they're consumed by Miette's error reporting. Targeted
// #[allow] attributes on the struct, impls and functions don't suppress this lint, so we must
// allow it at the module level.
#![allow(unused_assignments)]

use std::fmt;

use miette::{Diagnostic, NamedSource, SourceSpan};
use topiary_tree_sitter_facade::Range;

use crate::tree_sitter::NodeSpan;

#[derive(Diagnostic, Debug)]
pub(super) struct ErrorSpan {
    #[source_code]
    src: NamedSource<String>,

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

impl From<&Box<NodeSpan>> for ErrorSpan {
    fn from(span: &Box<NodeSpan>) -> Self {
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
