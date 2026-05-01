// The ErrorSpan struct uses Miette's Diagnostic derive macro, which reads fields through
// procedural macro attributes (#[source_code], #[label]). Newer versions of Clippy flag these
// field assignments as unused, even though they're consumed by Miette's error reporting. Targeted
// #[allow] attributes on the struct, impls and functions don't suppress this lint, so we must
// allow it at the module level.
#![allow(unused_assignments)]

use std::{
    boxed::Box,
    fmt,
    iter::Iterator,
    option::Option,
    path::{Path, PathBuf},
};

use miette::{LabeledSpan, MietteError, MietteSpanContents, SourceCode, SourceSpan, SpanContents};
use rootcause::markers::ObjectMarkerFor;
use topiary_tree_sitter_facade::Range;

/// ErrorSpan is meant to represent error codes that lives outside of the topiary
/// call stack and is rendered with [`miette::Report`].
/// Examples of files that generate ErrorSpans (these are typically runtime objects):
/// * configuration files (such as languages.ncl)
/// * code that is being formatted
/// * tree-sitter query files
#[derive(Debug, Default, Clone)]
pub struct ErrorSpan {
    source: Option<String>,
    // original
    filepath: Option<PathBuf>,
    language: Option<&'static str>,
    pub(crate) range: Option<Range>,

    // label for our immediate `SourceSpan`
    pub primary_label: Option<String>,
    span: Option<SourceSpan>,
}

impl miette::Diagnostic for ErrorSpan {
    #[allow(unused_variables)]
    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        self.span
            .map(|s| std::iter::once(LabeledSpan::new_with_span(Some(self.primary_label()), s)))
            .map(Box::new)
            .map(|b| b as Box<dyn Iterator<Item = LabeledSpan>>)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
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

    fn primary_label(&self) -> String {
        self.primary_label
            .clone()
            .unwrap_or_else(|| "(ERROR) node".to_owned())
    }
}

impl SourceCode for ErrorSpan {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        let inner_contents = self.source.as_deref().unwrap_or_default().read_span(
            span,
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

impl std::fmt::Display for ErrorSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(range) = self.range {
            let start = range.start_point();
            let end = range.end_point();
            // `QueryError`s and `Node`s report rows starting with 0
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

/// Defines methods that allow for attachment of metadata
/// that allow for rendering of the error location for use
/// in libraries such as `miette`.
pub trait SpanAttachment {
    fn attach_filepath<'a>(self, filepath: impl Into<Option<&'a Path>>) -> Self;
    fn attach_source<'a>(self, source: impl Into<Option<&'a str>>) -> Self;
    fn attach_language(self, language: impl Into<Option<&'static str>>) -> Self;
    fn attach_range(self, range: Range) -> Self;
    fn get_span(&mut self) -> Option<&mut ErrorSpan>;
}

impl<C, T> SpanAttachment for rootcause::Report<C, rootcause::markers::Mutable, T>
where
    ErrorSpan: ObjectMarkerFor<T>,
{
    fn attach_filepath<'a>(mut self, filepath: impl Into<Option<&'a Path>>) -> Self {
        let Some(filepath) = filepath.into() else {
            return self;
        };
        if let Some(span) = self.get_span() {
            span.filepath = Some(filepath.to_owned());
            return self;
        }
        self.attach(ErrorSpan::default().with_filepath(filepath))
    }

    fn attach_source<'a>(mut self, source: impl Into<Option<&'a str>>) -> Self {
        let Some(source) = source.into() else {
            return self;
        };
        if let Some(span) = self.get_span() {
            span.source = Some(source.to_owned());
            return self;
        }
        self.attach(ErrorSpan::default().with_source(source))
    }

    fn attach_language(mut self, language: impl Into<Option<&'static str>>) -> Self {
        let Some(language) = language.into() else {
            return self;
        };
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
    fn attach_filepath<'a>(self, filepath: impl Into<Option<&'a Path>>) -> Self {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.attach_filepath(filepath)),
        }
    }

    fn attach_source<'a>(self, source: impl Into<Option<&'a str>>) -> Self {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.attach_source(source)),
        }
    }

    fn attach_language(self, language: impl Into<Option<&'static str>>) -> Self {
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
