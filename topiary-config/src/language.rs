//! This module contains the `Language` struct, which represents a language configuration, and
//! associated methods.

use std::collections::{HashMap, HashSet};
#[cfg(all(test, not(target_arch = "wasm32")))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use crate::error::TopiaryConfigResult;
#[cfg(not(target_arch = "wasm32"))]
use crate::error::{TopiaryConfigError, TopiaryConfigFetchingError};

#[cfg(not(target_arch = "wasm32"))]
pub use tree_sitter_graft::{GitSource, Grammar, GrammarSource, Loader as GrammarLoader};

/// Language definitions, as far as the CLI and configuration are concerned, contain everything
/// needed to configure formatting for that language.
#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
pub struct Language {
    /// The name of the language, used as a key when looking up information in the deserialised
    /// configuration and to convert to the respective Tree-sitter grammar
    pub name: String,

    /// The configuration of the language, includes all properties that Topiary
    /// needs to properly format the language
    pub config: LanguageConfiguration,
}

#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
pub struct LanguageConfiguration {
    /// A set of the filetype extensions associated with this language. This enables Topiary to
    /// switch to the right language based on the input filename.
    pub extensions: HashSet<String>,

    /// The indentation string used for this language; defaults to "  " (i.e., two spaces). Any
    /// string can be provided, but in most instances it will be some whitespace (e.g., "    ",
    /// "\t", etc.)
    pub indent: Option<String>,

    /// The tree-sitter source of the language, contains all that is needed to pull and compile the tree-sitter grammar
    pub grammar: Grammar,

    /// Optional map of named queries (e.g. `formatting`, `injections`). When present, entries
    /// override the disk-search chain in `find_query_file`.
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(default)]
    // TODO Query source
    pub queries: Option<HashMap<String, Query>>,
}

#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
#[cfg(target_arch = "wasm32")]
pub struct Grammar {
    pub symbol: Option<String>,
}

/// A query file location. Either a local `path`, or a `path` inside a git checkout that
/// Topiary will fetch and cache on demand.
#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct QuerySource {
    /// Optional git source; when present, `path` is resolved relative to the checkout root.
    pub git: Option<GitSource>,
    /// Path to the query file (relative to the git checkout root when `git` is set,
    /// otherwise resolved as-is).
    pub path: PathBuf,
}

/// A named query entry (e.g. `formatting`, `injections`). The Nickel contract is
/// non-exhaustive so this is a struct rather than a tuple around `QuerySource` to allow
/// future per-query metadata.
#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct Query {
    pub source: QuerySource,
}

impl Language {
    pub fn new(name: String, config: LanguageConfiguration) -> Self {
        Self { name, config }
    }

    pub fn indent(&self) -> Option<String> {
        self.config.indent.clone()
    }

    /// Look up a named `Query` entry (e.g. "formatting", "injections") on this language's config.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn config_query(&self, query_name: &str) -> Option<&Query> {
        self.config.queries.as_ref()?.get(query_name)
    }

    /// Resolve a [`QuerySource`] to an on-disk path
    #[cfg(not(target_arch = "wasm32"))]
    pub fn resolve_query_path(
        &self,
        source: &QuerySource,
    ) -> Result<PathBuf, TopiaryConfigFetchingError> {
        let loader = GrammarLoader::for_project("topiary")?;
        self.resolve_query_path_with(source, &loader)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn resolve_query_path_with(
        &self,
        source: &QuerySource,
        loader: &GrammarLoader,
    ) -> Result<PathBuf, TopiaryConfigFetchingError> {
        let Some(git) = source.git.as_ref() else {
            return Ok(source.path.clone());
        };

        let checkout = loader.source_cache().checkout(git)?;
        Ok(checkout.path().join(&source.path))
    }

    /// Locate a query file for this language by well-known name (e.g. `"formatting"`,
    /// `"injections"`, matching the constants exported by `topiary-queries`).
    ///
    /// Prefer `languages.<language>.<query_name>` config entries over implicit query paths.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn find_query_file(&self, query_name: &str) -> TopiaryConfigResult<PathBuf> {
        let loader = GrammarLoader::for_project("topiary")
            .map_err(TopiaryConfigFetchingError::from)
            .map_err(TopiaryConfigError::Fetching)?;
        self.find_query_file_with(query_name, &loader)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn find_query_file_with(
        &self,
        query_name: &str,
        loader: &GrammarLoader,
    ) -> TopiaryConfigResult<PathBuf> {
        use crate::source::Source;

        let language_name = self.name.as_str();

        if let Some(query) = self.config_query(query_name) {
            let path = self
                .resolve_query_path_with(&query.source, loader)
                .map_err(TopiaryConfigError::Fetching)?;
            if path.is_file() {
                log::debug!(
                    "detected  {language_name}.{query_name} query at {}",
                    path.display()
                );
                return Ok(path);
            }
            return Err(TopiaryConfigError::QueryFileNotFound(path));
        }

        #[rustfmt::skip]
        let potentials: [Option<PathBuf>; 5] = [
            std::env::var("TOPIARY_LANGUAGE_DIR").map(PathBuf::from).ok(),
            option_env!("TOPIARY_LANGUAGE_DIR").map(PathBuf::from),
            Source::fetch_one(None).queries_dir(),
            Some(PathBuf::from("./topiary-queries/queries")),
            Some(PathBuf::from("../topiary-queries/queries")),
        ];

        let path_match = potentials
            .into_iter()
            .flatten()
            .flat_map(|path| {
                let mut paths = vec![
                    // New layout: <dir>/<lang>/<query_name>.scm
                    path.join(language_name).join(format!("{query_name}.scm")),
                ];
                if query_name == topiary_queries::FORMATTING_QUERY {
                    // Old layout: <dir>/<lang>.scm
                    paths.push(path.join(format!("{language_name}.scm")));
                }
                paths
            })
            .find(|path| {
                log::trace!("checking if {} exists", path.display());
                path.exists()
            })
            .ok_or_else(|| TopiaryConfigError::QueryFileNotFound(PathBuf::from(language_name)))?;

        // handle old formatting filepath warning here
        if query_name == topiary_queries::FORMATTING_QUERY {
            let lang_file = format!("{language_name}.scm");
            if path_match.ends_with(&lang_file) {
                log::warn!("deprecated formatter file: {lang_file}
formatting queries with '<language_name>.scm' filenames deprecated and will not be searched for in a future release"
                );
            }
        }
        Ok(path_match)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn grammar(
        &self,
    ) -> Result<topiary_tree_sitter_facade::Language, TopiaryConfigFetchingError> {
        let loader = GrammarLoader::for_project("topiary")?;
        self.grammar_with(&loader)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn grammar_with(
        &self,
        loader: &GrammarLoader,
    ) -> Result<topiary_tree_sitter_facade::Language, TopiaryConfigFetchingError> {
        loader
            .load(&self.name, &self.config.grammar)
            .map(Into::into)
            .map_err(Into::into)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn grammar(&self) -> TopiaryConfigResult<topiary_tree_sitter_facade::Language> {
        let language_name = self.name.as_str();

        let grammar_path = if language_name == "tree_sitter_query" {
            "/playground/scripts/tree-sitter-query.wasm".to_string()
        } else {
            format!("/playground/scripts/tree-sitter-{language_name}.wasm")
        };

        Ok(
            topiary_web_tree_sitter_sys::Language::load_path(&grammar_path)
                .await
                .map_err(|e| {
                    let error: topiary_tree_sitter_facade::LanguageError = e.into();
                    error
                })?
                .into(),
        )
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use nickel_lang_core::deserialize::from_str as from_nickel_str;

    #[test]
    fn language_local_sources() {
        let src = r#"
{
  extensions = ["ncl"],
  grammar.source.library_path.path = "/tmp/grammar.so",
  queries.formatting.source.path = "/path/to/nickel/formatting.scm",
}
        "#;
        let config: LanguageConfiguration = from_nickel_str(src).unwrap();

        assert!(matches!(
            &config.grammar,
            Grammar {
                source: GrammarSource::LibraryPath { path: p },
                symbol: None,
                ..
             }
             if p == Path::new("/tmp/grammar.so")
        ));

        let formatting = config.queries.unwrap().get("formatting").cloned().unwrap();
        assert!(matches!(
            &formatting,
            Query { source: QuerySource { git: None, path } } if path == Path::new("/path/to/nickel/formatting.scm")
        ));
    }

    #[test]
    fn language_git_sources() {
        let src = r#"
{
  extensions | default = ["md"],
  grammar.source | default = {
    git = {
      url = "https://github.com/tree-sitter-grammars/tree-sitter-markdown.git",
      rev = "c3570720f7f7bbad22fe96603f106276618e0cf5",
      subdir = "tree-sitter-markdown",
      nixHash = "sha256-wQKcqU0V6gHj84qOkUwdXsBW3f6MNfJMFxuGTucAgh8=",
    },
  },
  queries = {
    formatting.source = {
      git = {
        url = "https://github.com/topiary/topiary.git",
        rev = "d2c79b9ecd341d40aa0baf87f4a761ae242dfa67",
      },
      path = "topiary-queries/queries/markdown/formatting.scm"
    },
    injections.source = {
      git = formatting.source.git,
      path = "topiary-queries/queries/markdown/injections.scm",
    },
  }
}
        "#;
        let config: LanguageConfiguration = from_nickel_str(src).unwrap();

        assert!(config.extensions.contains("md"));

        let expected_git = GitSource::new(
            "https://github.com/tree-sitter-grammars/tree-sitter-markdown.git",
            "c3570720f7f7bbad22fe96603f106276618e0cf5",
        );
        assert!(matches!(
            config.grammar.source,
            GrammarSource::Git {
                source,
                subdir: Some(p),
            } if source == expected_git && p == Path::new("tree-sitter-markdown")
        ));

        let queries = config.queries.unwrap();

        let expected_git = GitSource::new(
            "https://github.com/topiary/topiary.git",
            "d2c79b9ecd341d40aa0baf87f4a761ae242dfa67",
        );
        let formatting = queries.get("formatting").unwrap();
        assert!(matches!(
            formatting,
            Query {
                source: QuerySource { git: Some(git), path }
            } if git == &expected_git && path.ends_with("formatting.scm")
        ));

        let injections = queries.get("injections").unwrap();
        assert!(matches!(
            injections,
            Query {
                source: QuerySource { git: Some(git), path }
            } if git == &expected_git && path.ends_with("injections.scm")
        ));
    }

    #[test]
    fn languages_merge_built_queries() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let config_file = tmp_dir.path().join("languages.ncl");
        std::fs::write(
            &config_file,
            r#"{ languages.markdown.queries.formatting.source.path = "/tmp/formatting.scm" }"#,
        )
        .unwrap();

        let (config, _) = crate::Configuration::fetch(false, Some(config_file.as_path())).unwrap();
        let formatting = config
            .get_language_cfg("markdown")
            .unwrap()
            .config_query("formatting")
            .unwrap();

        assert!(matches!(
            &formatting.source,
            QuerySource { git: None, path } if path == Path::new("/tmp/formatting.scm")
        ));
    }

    #[test]
    fn grammar_symbol_override() {
        let src = r#"
{
  source = { library_path = { path = "/tmp/grammar.so" } },
  symbol = "tree_sitter_query"
}
        "#;
        let grammar: Grammar = from_nickel_str(src).unwrap();
        assert_eq!(grammar.symbol.as_deref(), Some("tree_sitter_query"));
        assert_eq!(
            grammar.source,
            GrammarSource::library_path(PathBuf::from("/tmp/grammar.so"))
        );
    }

    #[test]
    fn source_path_uses_graft_shape() {
        let source: GrammarSource =
            from_nickel_str(r#"{ source_path = { path = "/tmp/json", subdir = "grammar" } }"#)
                .unwrap();
        assert_eq!(
            source,
            GrammarSource::source_path("/tmp/json", Some(PathBuf::from("grammar")))
        );
    }

    #[test]
    fn legacy_grammar_source_is_rejected() {
        assert!(from_nickel_str::<GrammarSource>(r#"{ path = "/tmp/grammar.so" }"#).is_err());
        assert!(
            from_nickel_str::<GrammarSource>(
                r#"{ git = { git = "https://example.invalid", rev = "main" } }"#
            )
            .is_err()
        );
    }
}
