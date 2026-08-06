#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub type LogType = tree_sitter::LogType;

    pub type Logger<'a> = Box<dyn FnMut(LogType, &str) + 'a + Send>;

    pub struct LoggerReturn<'a, 's> {
        #[allow(clippy::borrowed_box)]
        pub inner: &'s Logger<'a>,
    }

    impl<'a, 's> LoggerReturn<'a, 's> {
        #[allow(clippy::borrowed_box)]
        #[inline]
        pub(crate) fn new(inner: &'s Logger<'a>) -> Self {
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
        pub inner: Logger<'a>,
        phantom: std::marker::PhantomData<&'s ()>,
    }

    impl<'a, 's> LoggerReturn<'a, 's> {
        #[inline]
        pub(crate) fn new(inner: Logger<'a>) -> Self {
            let phantom = std::marker::PhantomData;
            Self { inner, phantom }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
