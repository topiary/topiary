// The ErrorSpan struct uses Miette's Diagnostic derive macro, which reads fields through
// procedural macro attributes (#[source_code], #[label]). Newer versions of Clippy flag these
// field assignments as unused, even though they're consumed by Miette's error reporting. Targeted
// #[allow] attributes on the struct, impls and functions don't suppress this lint, so we must
// allow it at the module level.
#![allow(unused_assignments)]

use std::{
    any::TypeId,
    fmt,
    path::{Path, PathBuf},
};

use miette::{
    Diagnostic, MietteError, MietteSpanContents, NamedSource, SourceCode, SourceSpan, SpanContents,
};
use rootcause::{
    markers::{self, ObjectMarkerFor},
    report_attachment::{ReportAttachment, ReportAttachmentMut},
};
use topiary_tree_sitter_facade::Range;

use crate::tree_sitter::NodeSpan;

#[derive(Diagnostic, Debug, Default)]
pub(super) struct ErrorSpan {
    source: Option<String>,
    filepath: Option<PathBuf>,
    language: Option<&'static str>,
    range: Option<Range>,

    #[label("(ERROR) node")]
    span: Option<SourceSpan>,
}

impl ErrorSpan {
    pub fn with_source(mut self, source: &str) -> Self {
        self.source = Some(source.to_owned());
        self
    }
    pub fn with_filepath(mut self, filepath: &Path) -> Self {
        self.filepath = Some(filepath.to_owned());
        self
    }

    pub fn set_language(&mut self, language: &'static str) {
        self.language = Some(language);
    }

    pub fn with_language(mut self, language: &'static str) -> Self {
        self.set_language(language);
        self
    }

    fn set_range(&mut self, range: Range) {
        self.range = Some(range);
        self.span = Some((range.start_byte() as usize..=range.end_byte() as usize).into());
    }

    /// Adds a [`SourceSpan`] from the [`Self`]'s byte range
    pub fn with_range(mut self, range: Range) -> Self {
        self.set_range(range);
        self
    }

    fn name(&self) -> &str {
        self.filepath
            .as_ref()
            .and_then(|f| f.to_str())
            .unwrap_or("built-in")
    }
}

impl SourceCode for ErrorSpan {
    fn read_span<'a>(
        &'a self,
        _span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        let span = self.span.unwrap_or(0.into());
        let inner_contents = self.source.as_deref().unwrap_or_default().read_span(
            &span,
            context_lines_before,
            context_lines_after,
        )?;
        let mut contents = MietteSpanContents::new_named(
            self.name().to_owned(),
            inner_contents.data(),
            *inner_contents.span(),
            inner_contents.line(),
            inner_contents.column(),
            inner_contents.line_count(),
        );
        if let Some(language) = self.language {
            contents = contents.with_language(language);
        }
        Ok(Box::new(contents))
    }
}

// impl miette::Diagnostic for ErrorSpan {
// #[allow(unused_variables)]
// fn labels(
//     &self,
// ) -> std::option::Option<
//     std::boxed::Box<dyn std::iter::Iterator<Item = miette::LabeledSpan> + '_>,
// > {
//     use miette::macro_helpers::ToOption;
//     let Self { src, span } = self;
//     let labels_iter = vec![
//         miette::macro_helpers::OptionalWrapper::<SourceSpan>::new()
//             .to_option(&self.span)
//             .map(|__miette_internal_var| {
//                 miette::LabeledSpan::new_with_span(
//                     std::option::Option::Some(format!("(ERROR) node")),
//                     __miette_internal_var.clone(),
//                 )
//             }),
//     ]
//     .into_iter();
//     std::option::Option::Some(Box::new(
//         labels_iter.filter(Option::is_some).map(Option::unwrap),
//     ))
// }
//     #[allow(unused_variables)]
//     fn source_code(&self) -> Option<&dyn miette::SourceCode> {
//         let Self { src, span } = self;
//         self.src.as_ref().map(|s| s as _)
//     }
// }

impl std::fmt::Display for ErrorSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(range) = self.range {
            let start = range.start_point();
            let end = range.end_point();
            write!(
                f,
                "Parsing error between line {}, column {} and line {}, column {}",
                start.row(),
                start.column(),
                end.row(),
                end.column()
            )
        } else {
            write!(f, "Parsing error: missing range",)
        }
    }
}

impl std::error::Error for ErrorSpan {}

// impl From<&Box<NodeSpan>> for ErrorSpan {
//     fn from(span: &Box<NodeSpan>) -> Self {
//         Self {
//             source: NamedSource::new(
//                 span.location.clone().unwrap_or_default(),
//                 span.content.clone().unwrap_or_default(),
//             )
//             .with_language(span.language),
//             span: span.source_span(),
//             range: span.range,
//         }
//     }
// }

pub trait SpanAttachment {
    fn attach_filepath(self, filepath: &Path) -> Self;
    fn attach_source(self, source: &str) -> Self;
    fn attach_language(self, language: &'static str) -> Self;
    fn attach_range(self, span: Range) -> Self;
    fn get_span(&mut self) -> Option<&mut ErrorSpan>;
}

impl<C, T> SpanAttachment for rootcause::Report<C, rootcause::markers::Mutable, T>
where
    ErrorSpan: ObjectMarkerFor<T>,
{
    fn attach_filepath(mut self, filepath: &Path) -> Self {
        if let Some(span) = self.get_span() {
            span.filepath = Some(filepath.to_owned());
            return self;
        }
        self.attach(ErrorSpan::default().with_filepath(filepath))
    }

    fn attach_source(mut self, source: &str) -> Self {
        if let Some(span) = self.get_span() {
            span.source = Some(source.to_owned());
            return self;
        }
        self.attach(ErrorSpan::default().with_source(source))
    }

    fn attach_language(mut self, language: &'static str) -> Self {
        if let Some(span) = self.get_span() {
            span.language = Some(language);
            return self;
        }
        self.attach(ErrorSpan::default().with_language(language))
    }

    fn attach_range(mut self, range: Range) -> Self {
        if let Some(span) = self.get_span() {
            span.set_range(range);
            return self;
        }
        self.attach(ErrorSpan::default().with_range(range))
    }

    fn get_span(&mut self) -> Option<&mut ErrorSpan> {
        let attachments = self.attachments_mut();
        attachments
            .iter_mut()
            .find_map(|a| a.downcast_attachment::<ErrorSpan>().ok())
            .map(|a| a.into_inner_mut())
    }
}
