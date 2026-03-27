use std::{error, fmt, io, path, result};

pub type TopiaryConfigResult<T> = result::Result<T, TopiaryConfigError>;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum TopiaryConfigError {
    FileNotFound(path::PathBuf),
    UnknownLanguage(String),
    UnknownExtension(String),
    NoExtension(path::PathBuf),
    #[cfg(not(target_arch = "wasm32"))]
    QueryFileNotFound(path::PathBuf),
    Io(io::Error),
    Missing,
    TreeSitterFacade(topiary_tree_sitter_facade::LanguageError),
    Nickel(Box<nickel_lang_core::error::Error>),
    NickelDeserialization(nickel_lang_core::deserialize::RustDeserializationError),
    #[cfg(not(target_arch = "wasm32"))]
    Fetching(TopiaryConfigFetchingError),
}

#[derive(Debug)]
/// Topiary can fetch an compile grammars, doing so may create errors.
/// Usually, this error would be part of the `TopiaryConfigError`, however, that enum also includes `nickel_lang_core::error::Error`, which does not implement Sync/Send.
/// Since fetching an compilation is something that can easily be parallelized, we create a special error that DOES implement Sync/Send.
#[cfg(not(target_arch = "wasm32"))]
pub enum TopiaryConfigFetchingError {
    Git(anyhow::Error),
    Build(tree_sitter_loader::LoaderError),
    Io(io::Error),
    LibLoading(libloading::Error),
    GrammarFileNotFound(path::PathBuf),
}

impl fmt::Display for TopiaryConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopiaryConfigError::FileNotFound(path) => write!(
                f,
                "We tried to find your configuration file at {}, but failed to do so. Make sure the file exists.",
                path.display()
            ),
            TopiaryConfigError::UnknownLanguage(lang) => write!(
                f,
                "You were looking for language \"{lang}\", but we do not know that language."
            ),
            TopiaryConfigError::UnknownExtension(ext) => write!(
                f,
                "You tried to format a file with extension: \"{ext}\", but we do not know that extension. Make sure the extension is in your configuration file!"
            ),
            TopiaryConfigError::NoExtension(path) => write!(
                f,
                "You tried to format {} without specifying a language, but we cannot automatically detect the language because we can't find the filetype extension.",
                path.display()
            ),
            #[cfg(not(target_arch = "wasm32"))]
            TopiaryConfigError::QueryFileNotFound(path) => write!(
                f,
                "We could not find the query file: \"{}\" anywhere. If you use the TOPIARY_LANGUAGE_DIR environment variable, make sure it set set correctly.",
                path.display()
            ),
            TopiaryConfigError::Io(error) => write!(f, "We encountered an io error: {error}"),
            TopiaryConfigError::Missing => write!(
                f,
                "A configuration file is missing. If you passed a configuration file, make sure it exists."
            ),
            TopiaryConfigError::TreeSitterFacade(_) => {
                write!(f, "We could not load the grammar for the given language")
            }
            TopiaryConfigError::Nickel(e) => write!(
                f,
                "Nickel error: {e:#?}\n\nDid you forget to add a \"priority\" annotation in your config file?"
            ),
            TopiaryConfigError::NickelDeserialization(e) => write!(f, "Nickel error: {e:#?}"),
            #[cfg(not(target_arch = "wasm32"))]
            TopiaryConfigError::Fetching(e) => write!(f, "Error Fetching Language: {e}"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl fmt::Display for TopiaryConfigFetchingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopiaryConfigFetchingError::Git(e) => write!(f, "Git error: {e:?}"),
            TopiaryConfigFetchingError::Build(e) => {
                write!(f, "Compilation error: {e},")
            }
            TopiaryConfigFetchingError::Io(error) => {
                write!(f, "We encountered an io error: {error}")
            }
            TopiaryConfigFetchingError::LibLoading(e) => write!(f, "Libloading error: {e:?}"),
            TopiaryConfigFetchingError::GrammarFileNotFound(path) => write!(
                f,
                "Attempted to load grammar at `{}`, but no file found",
                path.display()
            ),
        }
    }
}

impl From<nickel_lang_core::deserialize::RustDeserializationError> for TopiaryConfigError {
    fn from(e: nickel_lang_core::deserialize::RustDeserializationError) -> Self {
        Self::NickelDeserialization(e)
    }
}

impl From<nickel_lang_core::error::Error> for TopiaryConfigError {
    fn from(e: nickel_lang_core::error::Error) -> Self {
        Self::Nickel(e.into())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<TopiaryConfigFetchingError> for TopiaryConfigError {
    fn from(e: TopiaryConfigFetchingError) -> Self {
        Self::Fetching(e)
    }
}

impl From<io::Error> for TopiaryConfigError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<io::Error> for TopiaryConfigFetchingError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<topiary_tree_sitter_facade::LanguageError> for TopiaryConfigError {
    fn from(e: topiary_tree_sitter_facade::LanguageError) -> Self {
        Self::TreeSitterFacade(e)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<libloading::Error> for TopiaryConfigFetchingError {
    fn from(e: libloading::Error) -> Self {
        Self::LibLoading(e)
    }
}

impl error::Error for TopiaryConfigError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            TopiaryConfigError::Io(e) => e.source(),
            _ => None,
        }
    }
}
