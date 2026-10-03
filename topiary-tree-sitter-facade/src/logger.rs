#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub type LogType = tree_sitter::LogType;

    // Since tree-sitter 0.27, `Parser::set_logger` only accepts `Send` loggers;
    // borrowing, non-`Send` callbacks are relegated to the unsafe
    // `Parser::set_logger_unchecked`, which we do not expose.
    pub type Logger<'a> = Box<dyn FnMut(LogType, &str) + Send + 'a>;

    pub struct LoggerReturn<'a, 's> {
        #[allow(clippy::borrowed_box, clippy::type_complexity)]
        pub inner: &'s Box<dyn FnMut(LogType, &str) + Send + 'a>,
    }

    impl<'a, 's> LoggerReturn<'a, 's> {
        #[allow(clippy::borrowed_box, clippy::type_complexity)]
        #[inline]
        pub(crate) fn new(inner: &'s Box<dyn FnMut(LogType, &str) + Send + 'a>) -> Self {
            Self { inner }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use js_sys::JsString;

    pub type LogType = JsString;

    pub type Logger<'a> = Box<dyn FnMut(LogType, JsString) + 'a>;

    pub struct LoggerReturn<'a, 's> {
        pub inner: Box<dyn FnMut(LogType, JsString) + 'a>,
        phantom: std::marker::PhantomData<&'s ()>,
    }

    impl<'a, 's> LoggerReturn<'a, 's> {
        #[inline]
        pub(crate) fn new(inner: Box<dyn FnMut(LogType, JsString) + 'a>) -> Self {
            let phantom = std::marker::PhantomData;
            Self { inner, phantom }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
