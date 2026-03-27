#[cfg(not(target_arch = "wasm32"))]
mod native {
    use miette::{SourceOffset, SourceSpan};

    use crate::Point;

    #[derive(Eq, PartialEq)]
    pub struct IncludedRangesError {
        pub(crate) inner: tree_sitter::IncludedRangesError,
    }

    impl std::fmt::Debug for IncludedRangesError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            std::fmt::Debug::fmt(&self.inner, fmt)
        }
    }

    impl std::fmt::Display for IncludedRangesError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(fmt, "{:?}", self.inner)
        }
    }

    impl std::error::Error for IncludedRangesError {}

    impl From<tree_sitter::IncludedRangesError> for IncludedRangesError {
        #[inline]
        fn from(inner: tree_sitter::IncludedRangesError) -> Self {
            Self { inner }
        }
    }

    unsafe impl Send for IncludedRangesError {}

    unsafe impl Sync for IncludedRangesError {}

    #[derive(Eq, PartialEq)]
    pub struct QueryError {
        pub inner: tree_sitter::QueryError,
    }

    impl std::fmt::Debug for QueryError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            std::fmt::Debug::fmt(&self.inner, fmt)
        }
    }

    impl std::fmt::Display for QueryError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(fmt, "{:?}", self.inner)
        }
    }

    impl std::error::Error for QueryError {}

    impl From<tree_sitter::QueryError> for QueryError {
        #[inline]
        fn from(inner: tree_sitter::QueryError) -> Self {
            Self { inner }
        }
    }

    impl QueryError {
        pub fn span(&self, content: &str) -> SourceSpan {
            let (row, col) = (self.inner.row, self.inner.column);
            // [tree_sitter::QueryError] only provides the linewise start of
            // the problematic query and thus we need the original content to find
            // where the line ends
            let row_len = content.lines().nth(row - 1).map(|l| l.len()).unwrap_or(1);
            let offset = SourceOffset::from_location(content, row, col);
            SourceSpan::new(offset, row_len)
        }
    }

    unsafe impl Send for QueryError {}

    unsafe impl Sync for QueryError {}

    #[derive(Eq, PartialEq)]
    pub struct LanguageError {
        pub(crate) inner: tree_sitter::LanguageError,
    }

    impl std::fmt::Debug for LanguageError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            std::fmt::Debug::fmt(&self.inner, fmt)
        }
    }

    impl std::fmt::Display for LanguageError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            std::fmt::Display::fmt(&self.inner, fmt)
        }
    }

    impl std::error::Error for LanguageError {}

    impl From<tree_sitter::LanguageError> for LanguageError {
        #[inline]
        fn from(inner: tree_sitter::LanguageError) -> Self {
            Self { inner }
        }
    }

    unsafe impl Send for LanguageError {}

    unsafe impl Sync for LanguageError {}

    #[derive(Eq, PartialEq)]
    pub struct ParserError {
        pub(crate) inner: std::convert::Infallible,
    }

    impl std::fmt::Debug for ParserError {
        fn fmt(&self, _fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            unreachable!()
        }
    }

    impl std::fmt::Display for ParserError {
        fn fmt(&self, _fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            unreachable!()
        }
    }

    impl std::error::Error for ParserError {}

    unsafe impl Send for ParserError {}

    unsafe impl Sync for ParserError {}
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
mod wasm {
    #[derive(Eq, PartialEq)]
    pub struct IncludedRangesError {
        pub(crate) inner: js_sys::Error,
    }

    unsafe impl Send for IncludedRangesError {}

    unsafe impl Sync for IncludedRangesError {}

    impl std::fmt::Debug for IncludedRangesError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            std::fmt::Debug::fmt(&self.inner, fmt)
        }
    }

    impl std::fmt::Display for IncludedRangesError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            fmt.write_str(&<_ as Into<String>>::into(self.inner.message()))
        }
    }

    impl std::error::Error for IncludedRangesError {}

    impl From<js_sys::Error> for IncludedRangesError {
        #[inline]
        fn from(inner: js_sys::Error) -> Self {
            Self { inner }
        }
    }

    #[derive(Eq, PartialEq)]
    pub struct LanguageError {
        pub(crate) inner: topiary_web_tree_sitter_sys::LanguageError,
    }

    unsafe impl Send for LanguageError {}

    unsafe impl Sync for LanguageError {}

    impl std::fmt::Debug for LanguageError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            std::fmt::Debug::fmt(&self.inner, fmt)
        }
    }

    impl std::fmt::Display for LanguageError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            fmt.write_str(&<_ as Into<String>>::into(self.inner.message()))
        }
    }

    impl std::error::Error for LanguageError {}

    impl From<topiary_web_tree_sitter_sys::LanguageError> for LanguageError {
        #[inline]
        fn from(inner: topiary_web_tree_sitter_sys::LanguageError) -> Self {
            Self { inner }
        }
    }

    #[derive(Eq, PartialEq)]
    pub struct ParserError {
        pub(crate) inner: topiary_web_tree_sitter_sys::ParserError,
    }

    unsafe impl Send for ParserError {}

    unsafe impl Sync for ParserError {}

    impl std::fmt::Debug for ParserError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            std::fmt::Debug::fmt(&self.inner, fmt)
        }
    }

    impl std::fmt::Display for ParserError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            fmt.write_str(&<_ as Into<String>>::into(self.inner.message()))
        }
    }

    impl std::error::Error for ParserError {}

    impl From<topiary_web_tree_sitter_sys::ParserError> for ParserError {
        #[inline]
        fn from(inner: topiary_web_tree_sitter_sys::ParserError) -> Self {
            Self { inner }
        }
    }

    #[derive(Eq, PartialEq)]
    pub struct QueryError {
        pub(crate) inner: topiary_web_tree_sitter_sys::QueryError,
    }

    unsafe impl Send for QueryError {}

    unsafe impl Sync for QueryError {}

    impl std::fmt::Debug for QueryError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            std::fmt::Debug::fmt(&self.inner, fmt)
        }
    }

    impl std::fmt::Display for QueryError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            fmt.write_str(&<_ as Into<String>>::into(self.inner.message()))
        }
    }

    impl std::error::Error for QueryError {}

    impl From<topiary_web_tree_sitter_sys::QueryError> for QueryError {
        #[inline]
        fn from(inner: topiary_web_tree_sitter_sys::QueryError) -> Self {
            Self { inner }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
