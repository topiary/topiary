use std::fmt;

use rootcause::{
    ReportMut,
    handlers::{AttachmentFormattingPlacement, AttachmentFormattingStyle, FormattingFunction},
    hooks::{
        attachment_formatter::{AttachmentFormatterHook, AttachmentParent},
        report_creation::ReportCreationHook,
    },
    markers::{Dynamic, Local, ObjectMarkerFor, SendSync},
    report_attachment::ReportAttachmentRef,
};
use topiary_core::ErrorSpan;
use topiary_tree_sitter_facade::QueryError;

// Move verbose query diagnostics to appendix instead of cluttering inline
pub struct SpanFormatter;

// This allows the ErrorSpan
impl AttachmentFormatterHook<ErrorSpan> for SpanFormatter {
    fn display(
        &self,
        attachment: ReportAttachmentRef<'_, ErrorSpan>,
        _attachment_parent: Option<AttachmentParent<'_>>,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let error_span = attachment.inner().clone();
        let report = miette::Report::new(error_span);
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

pub struct SpanHook;

impl SpanHook {
    fn on_create<T>(mut report: ReportMut<'_, Dynamic, T>)
    where
        ErrorSpan: ObjectMarkerFor<T>,
    {
        if let Some(query_error) = report.downcast_current_context::<QueryError>() {
            // TODO add error_span.with_label(...) setter methods
            let mut span = ErrorSpan::default()
                .with_range(query_error.range)
                .with_language("tree_sitter_query");
            span.primary_label = Some(format!("{query_error}"));
            report.attachments_mut().push(span.into());
        }
    }
}

impl ReportCreationHook for SpanHook {
    fn on_local_creation(&self, report: ReportMut<'_, Dynamic, Local>) {
        Self::on_create(report);
    }

    fn on_sendsync_creation(&self, report: ReportMut<'_, Dynamic, SendSync>) {
        Self::on_create(report);
    }
}
