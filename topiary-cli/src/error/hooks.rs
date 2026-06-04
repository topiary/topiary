
use rootcause::{
    hooks::report_creation::ReportCreationHook,
    markers::{Dynamic, Local, ObjectMarkerFor, SendSync},
    ReportMut,
};
use topiary_core::ErrorSpan;
use topiary_tree_sitter_facade::QueryError;

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
