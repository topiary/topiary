// The ErrorSpan struct uses Miette's Diagnostic derive macro, which reads fields through
// procedural macro attributes (#[source_code], #[label]). Newer versions of Clippy flag these
// field assignments as unused, even though they're consumed by Miette's error reporting. Targeted
// #[allow] attributes on the struct, impls and functions don't suppress this lint, so we must
// allow it at the module level.
#![allow(unused_assignments)]

use std::{
    boxed::Box,
    fmt,
    ops::Deref,
    path::{Path, PathBuf},
};

use miette::{Diagnostic, MietteError, MietteSpanContents, SourceCode, SourceSpan, SpanContents};
use rootcause::{
    handlers::{AttachmentFormattingPlacement, AttachmentFormattingStyle, FormattingFunction},
    hooks::attachment_formatter::AttachmentFormatterHook,
    markers::{Dynamic, ObjectMarkerFor},
    prelude::ResultExt,
    report_attachment::ReportAttachmentRef,
};
use topiary_tree_sitter_facade::Range;

#[derive(Clone, Debug, Default)]
pub struct ErrorSpan {
    source: Option<String>,
    filepath: Option<PathBuf>,
    language: Option<&'static str>,
    pub(crate) range: Option<Range>,

    span: Option<SourceSpan>,
}

impl miette::Diagnostic for ErrorSpan {
    #[allow(unused_variables)]
    fn labels(
        &self,
    ) -> std::option::Option<Box<dyn std::iter::Iterator<Item = miette::LabeledSpan> + '_>> {
        use miette::macro_helpers::ToOption;
        let Self {
            source,
            filepath,
            language,
            range,
            span,
        } = self;
        let labels_iter = vec![
            miette::macro_helpers::OptionalWrapper::<Option<SourceSpan>>::new()
                .to_option(&self.span)
                .map(|__miette_internal_var| {
                    miette::LabeledSpan::new_with_span(
                        std::option::Option::Some(format!("(ERROR) node")),
                        __miette_internal_var.clone(),
                    )
                }),
        ]
        .into_iter();
        std::option::Option::Some(Box::new(
            labels_iter.filter(Option::is_some).map(Option::unwrap),
        ))
    }
    #[allow(unused_variables)]
    fn source_code(&self) -> std::option::Option<&dyn miette::SourceCode> {
        Some(self)
    }
}

impl ErrorSpan {
    pub fn set_source(&mut self, source: &str) {
        self.source = Some(source.to_owned());
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.set_source(source);
        self
    }

    pub fn set_filepath(&mut self, filepath: &Path) {
        self.filepath = Some(filepath.to_owned());
    }
    pub fn with_filepath(mut self, filepath: &Path) -> Self {
        self.set_filepath(filepath);
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
            dbg!(language);
            contents = contents.with_language(language);
        }
        Ok(Box::new(contents))
    }
}

impl std::fmt::Display for ErrorSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(range) = self.range {
            let start = range.start_point();
            let end = range.end_point();
            // `QuerryError`s and `Node`s report rows starting with 0
            write!(
                f,
                "Parsing error between line {}, column {} and line {}, column {}",
                start.row() + 1,
                start.column(),
                end.row() + 1,
                end.column()
            )
        } else {
            write!(f, "Parsing error: missing range",)
        }
    }
}

impl std::error::Error for ErrorSpan {}
pub trait SpanAttachment {
    fn attach_filepath(self, filepath: &Path) -> Self;
    fn attach_source(self, source: &str) -> Self;
    fn attach_language(self, language: &'static str) -> Self;
    fn attach_range(self, range: Range) -> Self;
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

impl<V, E> SpanAttachment for Result<V, E>
where
    E: SpanAttachment,
{
    fn attach_filepath(self, filepath: &Path) -> Self {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.attach_filepath(filepath)),
        }
    }

    fn attach_source(self, source: &str) -> Self {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.attach_source(source)),
        }
    }

    fn attach_language(self, language: &'static str) -> Self {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.attach_language(language)),
        }
    }

    fn attach_range(self, range: Range) -> Self {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.attach_range(range)),
        }
    }

    fn get_span(&mut self) -> Option<&mut ErrorSpan> {
        match self {
            Ok(_) => None,
            Err(e) => e.get_span(),
        }
    }
}

// Move verbose query diagnostics to appendix instead of cluttering inline
pub struct MietteSpanFormatter;

impl AttachmentFormatterHook<ErrorSpan> for MietteSpanFormatter {
    fn display(
        &self,
        attachment: ReportAttachmentRef<'_, ErrorSpan>,
        _attachment_parent: Option<rootcause::hooks::attachment_formatter::AttachmentParent<'_>>,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let report = miette::Report::new(attachment.inner().clone());
        write!(f, "{report:?}")
    }
    fn preferred_formatting_style(
        &self,
        _attachment: ReportAttachmentRef<'_, Dynamic>,
        formatting_function: FormattingFunction,
    ) -> AttachmentFormattingStyle {
        match formatting_function {
            // for display printing we want the full verbosity level and
            // put it in an appendix
            FormattingFunction::Display => AttachmentFormattingStyle {
                placement: AttachmentFormattingPlacement::Appendix {
                    appendix_name: "Error Span",
                },
                function: FormattingFunction::Display,
                priority: 0,
            },
            // debug printing of the attachment is more compact so
            // we put it inline but at the end
            FormattingFunction::Debug => AttachmentFormattingStyle {
                placement: AttachmentFormattingPlacement::Inline,
                function: FormattingFunction::Display,
                priority: -10,
            },
        }
    }
}
