//! Support for delegating formatting to an external formatter.
//!
//! Some languages already have an excellent native formatter (e.g. `rustfmt`,
//! `gofmt`, `prettier`). Rather than trying to compete with them, Topiary can
//! hand a language over to such a program instead of applying its tree-sitter
//! formatting query.
//!
//! `topiary-core` decides *when* a language is delegated -- including keeping
//! injection handling around it -- but not *how*: the caller supplies the
//! runner. A runner is just a function, so core never spawns a process itself
//! and stays independent of the host platform (notably the web playground).
//! The CLI, for instance, supplies a runner that spawns a configured command
//! and pipes the input through it.

use std::{fmt, sync::Arc};

use crate::FormatterResult;

/// The formatting function wrapped by an [`ExternalFormatter`].
type FormatFn = dyn Fn(&str) -> FormatterResult<String> + Send + Sync;

/// A formatter that delegates to something outside Topiary.
///
/// This is intentionally opaque: core treats the wrapped function as a black
/// box that formats the given input and returns the formatted result.
#[derive(Clone)]
pub struct ExternalFormatter(Arc<FormatFn>);

impl ExternalFormatter {
    /// Build an external formatter from a formatting function.
    ///
    /// The function must be cheap to share and callable from any thread, as
    /// language definitions are cached and used concurrently.
    pub fn new<F>(format: F) -> Self
    where
        F: Fn(&str) -> FormatterResult<String> + Send + Sync + 'static,
    {
        Self(Arc::new(format))
    }

    /// Format `input`, returning the formatted result.
    ///
    /// # Errors
    ///
    /// Propagates whatever the supplied runner returns.
    pub fn run(&self, input: &str) -> FormatterResult<String> {
        (self.0)(input)
    }
}

impl fmt::Debug for ExternalFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExternalFormatter(..)")
    }
}
