use std::{
    fmt,
    path::{Path, PathBuf},
};

use rootcause::{
    Report,
    markers::{Mutable, SendSync},
    prelude::ResultExt,
    report,
};
use rootcause_preformat::PreformatReportExt;
use topiary_config::{Configuration, language::LocalRepos};
use topiary_core::{InjectionQuery, Language, SpanAttachment, TopiaryQuery};

#[derive(Debug, Clone, Hash)]
pub enum QuerySource {
    Path(PathBuf),
    BuiltIn(String),
}

impl From<PathBuf> for QuerySource {
    fn from(path: PathBuf) -> Self {
        QuerySource::Path(path)
    }
}

impl From<&PathBuf> for QuerySource {
    fn from(path: &PathBuf) -> Self {
        QuerySource::Path(path.clone())
    }
}

impl From<&str> for QuerySource {
    fn from(string: &str) -> Self {
        QuerySource::BuiltIn(String::from(string))
    }
}

impl fmt::Display for QuerySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuerySource::Path(p) => write!(f, "{}", p.display()),
            QuerySource::BuiltIn(_) => write!(f, "built-in query"),
        }
    }
}

impl QuerySource {
    pub fn filepath(&self) -> Option<&Path> {
        match self {
            QuerySource::Path(p) => Some(p.as_path()),
            QuerySource::BuiltIn(_) => None,
        }
    }

    pub async fn get_content(&self) -> Result<String, std::io::Error> {
        let contents = match self {
            Self::Path(query) => tokio::fs::read_to_string(query).await?,
            Self::BuiltIn(contents) => contents.to_owned(),
        };
        Ok(contents)
    }

    pub fn get_content_sync(&self) -> Result<String, std::io::Error> {
        let contents = match self {
            Self::Path(query) => std::fs::read_to_string(query)?,
            Self::BuiltIn(contents) => contents.to_owned(),
        };
        Ok(contents)
    }
}

#[derive(Debug)]
pub enum ResolverError {
    UnsupportedLanguage(String),
    QueryNotFound(String),
    Io(std::io::Error),
    Parsing(String),
    Config(String),
}

impl fmt::Display for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnsupportedLanguage(name) => {
                write!(f, "The specified language is unsupported: {name}")
            }
            Self::QueryNotFound(name) => write!(f, "Query file not found for language: {name}"),
            Self::Io(_) => write!(f, "I/O Error"),
            Self::Parsing(msg) => write!(f, "Parsing Error: {msg}"),
            Self::Config(msg) => write!(f, "Configuration Error: {msg}"),
        }
    }
}

impl std::error::Error for ResolverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

pub type ResolverResult<T> = Result<T, Report<ResolverError, Mutable, SendSync>>;

pub fn builtin_query<T: AsRef<str> + fmt::Display>(name: T) -> ResolverResult<QuerySource> {
    match name.as_ref() {
        #[cfg(feature = "bash")]
        "bash" => Ok(topiary_queries::bash().into()),

        #[cfg(feature = "css")]
        "css" => Ok(topiary_queries::css().into()),

        #[cfg(feature = "json")]
        "json" => Ok(topiary_queries::json().into()),

        #[cfg(feature = "markdown")]
        "markdown" => Ok(topiary_queries::markdown().into()),

        #[cfg(feature = "menhir")]
        "menhir" => Ok(topiary_queries::menhir().into()),

        #[cfg(feature = "nickel")]
        "nickel" => Ok(topiary_queries::nickel().into()),

        #[cfg(feature = "ocaml")]
        "ocaml" => Ok(topiary_queries::ocaml().into()),

        #[cfg(feature = "ocaml_interface")]
        "ocaml_interface" => Ok(topiary_queries::ocaml_interface().into()),

        #[cfg(feature = "ocamllex")]
        "ocamllex" => Ok(topiary_queries::ocamllex().into()),

        #[cfg(feature = "openscad")]
        "openscad" => Ok(topiary_queries::openscad().into()),

        #[cfg(feature = "rust")]
        "rust" => Ok(topiary_queries::rust().into()),

        #[cfg(feature = "sdml")]
        "sdml" => Ok(topiary_queries::sdml().into()),

        #[cfg(feature = "toml")]
        "toml" => Ok(topiary_queries::toml().into()),

        #[cfg(feature = "tree_sitter_query")]
        "tree_sitter_query" => Ok(topiary_queries::tree_sitter_query().into()),

        #[cfg(feature = "wit")]
        "wit" => Ok(topiary_queries::wit().into()),

        name => Err(report!(ResolverError::UnsupportedLanguage(
            name.to_string()
        ))),
    }
}

pub fn builtin_injection_query<T: AsRef<str>>(name: T) -> Option<QuerySource> {
    match name.as_ref() {
        #[cfg(feature = "markdown")]
        "markdown" => Some(topiary_queries::markdown_injections().into()),

        #[cfg(feature = "menhir")]
        "menhir" => Some(topiary_queries::menhir_injections().into()),

        #[cfg(feature = "ocamllex")]
        "ocamllex" => Some(topiary_queries::ocamllex_injections().into()),

        #[cfg(feature = "rust")]
        "rust" => Some(topiary_queries::rust_injections().into()),

        _ => None,
    }
}

/// Resolve a query using the configured repositories and cache directory, with a
/// fallback to the queries included at compile time.
pub fn query_source_with(
    language: &topiary_config::language::Language,
    query_name: &str,
    repos: &LocalRepos,
    cache_dir: Option<&Path>,
) -> ResolverResult<QuerySource> {
    match language.find_query_file_with(query_name, repos, cache_dir) {
        Ok(path) => Ok(path.into()),
        Err(error) => {
            log::warn!(
                "No {query_name} query files found in any of the expected locations for {}. Falling back to compile-time included files.",
                language.name
            );
            let builtin = match query_name {
                topiary_queries::FORMATTING_QUERY => builtin_query(&language.name),
                topiary_queries::INJECTIONS_QUERY => builtin_injection_query(&language.name)
                    .ok_or_else(|| report!(ResolverError::QueryNotFound(language.name.clone()))),
                _ => Err(report!(ResolverError::QueryNotFound(language.name.clone()))),
            };
            builtin.map_err(|builtin_error| {
                let mut error = report!(error)
                    .preformat()
                    .context(ResolverError::Config(format!(
                        "{query_name} query for {}",
                        language.name
                    )));
                error
                    .children_mut()
                    .push(builtin_error.into_cloneable().into_dynamic());
                error
            })
        }
    }
}

pub fn query_for_language(
    language: &topiary_config::language::Language,
) -> ResolverResult<QuerySource> {
    query_source_with(
        language,
        topiary_queries::FORMATTING_QUERY,
        &LocalRepos::new(),
        None,
    )
}

pub fn injection_query_for_language(
    language: &topiary_config::language::Language,
) -> Option<QuerySource> {
    query_source_with(
        language,
        topiary_queries::INJECTIONS_QUERY,
        &LocalRepos::new(),
        None,
    )
    .ok()
}

pub async fn resolve_language_by_name<T: AsRef<str>>(
    config: &Configuration,
    name: T,
) -> ResolverResult<Language> {
    resolve_language_by_name_sync(config, name.as_ref())
}

pub fn resolve_language_by_name_sync<T: AsRef<str> + fmt::Display>(
    config: &Configuration,
    name: T,
) -> ResolverResult<Language> {
    resolve_language_by_name_with(config, name, &LocalRepos::new())
}

/// Resolve a language while reusing repository checkouts and respecting the
/// configuration's cache directory.
pub fn resolve_language_by_name_with<T: AsRef<str> + fmt::Display>(
    config: &Configuration,
    name: T,
    repos: &LocalRepos,
) -> ResolverResult<Language> {
    let config_language = config.get_language_cfg(name.as_ref()).map_err(|e| {
        report!(e)
            .preformat()
            .context(ResolverError::Config(name.to_string()))
    })?;
    let grammar = config_language
        .fetch_grammar_with(repos, config.cache_dir())
        .context(ResolverError::Config(name.to_string()))?;
    let query_source = query_source_with(
        config_language,
        topiary_queries::FORMATTING_QUERY,
        repos,
        config.cache_dir(),
    )?;
    let query_content = query_source
        .get_content_sync()
        .map_err(|e| report!(ResolverError::Io(e)))?;
    let formatting_query = TopiaryQuery::new(&grammar, &query_content)
        .attach_filepath(query_source.filepath())
        .context(ResolverError::Parsing("formatting query".to_owned()))?;
    let injection_query = match query_source_with(
        config_language,
        topiary_queries::INJECTIONS_QUERY,
        repos,
        config.cache_dir(),
    ) {
        Ok(source) => {
            let contents = source
                .get_content_sync()
                .map_err(|e| report!(ResolverError::Io(e)))?;
            Some(
                InjectionQuery::new(&grammar, &contents)
                    .attach_filepath(source.filepath())
                    .context(ResolverError::Parsing("injection query".to_owned()))?,
            )
        }
        Err(_) => None,
    };

    Ok(Language {
        name: name.as_ref().to_string(),
        formatting_query: Some(formatting_query),
        injection_query,
        grammar,
        indent: config_language.indent(),
    })
}
