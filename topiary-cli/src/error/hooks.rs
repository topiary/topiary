use std::fmt;

use rootcause::{
    handlers::{AttachmentFormattingPlacement, AttachmentFormattingStyle, FormattingFunction},
    hooks::attachment_formatter::{AttachmentFormatterHook, AttachmentParent},
    markers::Dynamic,
    report_attachment::ReportAttachmentRef,
};
use topiary_core::ErrorSpan;

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
