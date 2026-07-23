//! This module contains the `Language` struct, which represents a language configuration, and
//! associated methods.

#[cfg(not(target_arch = "wasm32"))]
use anyhow::anyhow;
#[cfg(not(target_arch = "wasm32"))]
use gix::{
    ObjectId,
    interrupt::IS_INTERRUPTED,
    progress::Discard,
    remote::{self, Direction, fetch, fetch::refmap},
    worktree::state::checkout,
};
#[cfg(not(target_arch = "wasm32"))]
use std::num::NonZero;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;
use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    path::Path,
};
#[cfg(not(target_arch = "wasm32"))]
use tempfile::TempDir;

use crate::error::TopiaryConfigResult;
#[cfg(not(target_arch = "wasm32"))]
use crate::error::{TopiaryConfigError, TopiaryConfigFetchingError};

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
pub struct Grammar {
    #[cfg(not(target_arch = "wasm32"))]
    pub source: GrammarSource,
    /// If symbol of the language in the compiled grammar. Usually this is
    /// `tree_sitter_<LANGUAGE_NAME>`, but in rare cases it differs. For
    /// instance our "tree-sitter-query" language, where the symbol is:
    /// `tree_sitter_query` instead of `tree_sitter_tree_sitter_query`.
    pub symbol: Option<String>,
}

#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub enum GrammarSource {
    #[serde(rename = "path")]
    Path { path: PathBuf },
    #[serde(rename = "git")]
    Git {
        #[serde(flatten)]
        git: GitSource,
        #[serde(default)]
        subdir: Option<PathBuf>,
    },
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

#[derive(Debug, serde::Deserialize, PartialEq, Eq, Hash, serde::Serialize, Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct GitSource {
    /// The URL of the git repository that contains the tree-sitter grammar.
    pub git: String,
    /// The revision of the git repository to use.
    pub rev: String,
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
        self.resolve_query_path_with(source, &LocalRepos::new())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn resolve_query_path_with(
        &self,
        source: &QuerySource,
        repos: &LocalRepos,
    ) -> Result<PathBuf, TopiaryConfigFetchingError> {
        let Some(git) = source.git.as_ref() else {
            return Ok(source.path.clone());
        };

        let checkout = repos.get_or_insert(git)?;
        Ok(checkout.join(&source.path))
    }

    /// Locate a query file for this language by well-known name (e.g. `"formatting"`,
    /// `"injections"`, matching the constants exported by `topiary-queries`).
    ///
    /// Resolution order:
    /// 1. A `queries.<query_name>` entry on this language's config, if present. When it points
    ///    at a git source the checkout is materialised under `<cache>/<lang>/queries/<rev>/`
    ///    on first use and reused thereafter.
    /// 2. The disk-search chain: `TOPIARY_LANGUAGE_DIR`, the config's `queries/` directory,
    ///    and the workspace-relative fallbacks.
    ///
    /// Returns `Err(QueryFileNotFound)` when both routes fail, so callers can decide whether
    /// to fall back to compile-time built-ins (formatting) or treat absence as fine
    /// (injections).
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::result_large_err)]
    pub fn find_query_file(&self, query_name: &str) -> TopiaryConfigResult<PathBuf> {
        self.find_query_file_with(query_name, &LocalRepos::new())
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::result_large_err)]
    pub fn find_query_file_with(
        &self,
        query_name: &str,
        repos: &LocalRepos,
    ) -> TopiaryConfigResult<PathBuf> {
        use crate::source::Source;

        let language_name = self.name.as_str();

        if let Some(query) = self.config_query(query_name) {
            let path = self
                .resolve_query_path_with(&query.source, repos)
                .map_err(TopiaryConfigError::Fetching)?;
            if path.is_file() {
                return Ok(path);
            }
            return Err(TopiaryConfigError::QueryFileNotFound(path));
        }

        #[rustfmt::skip]
        let potentials: [Option<PathBuf>; 5] = [
            std::env::var("TOPIARY_LANGUAGE_DIR").map(PathBuf::from).ok(),
            option_env!("TOPIARY_LANGUAGE_DIR").map(PathBuf::from),
            Source::fetch_one(&None).queries_dir(),
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
    // Returns the library path, and ensures the parent directories exist.
    pub fn library_path(&self) -> std::io::Result<PathBuf> {
        match &self.config.grammar.source {
            GrammarSource::Git { git, .. } => {
                let mut library_path = crate::project_dirs().cache_dir().to_path_buf();
                library_path.push(self.name.clone());
                std::fs::create_dir_all(&library_path)?;

                // Set the output path as the revision of the grammar,
                // with a platform-appropriate extension
                library_path.push(git.rev.clone());
                library_path.set_extension(std::env::consts::DLL_EXTENSION);

                Ok(library_path)
            }

            GrammarSource::Path { path } => Ok(path.to_path_buf()),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    // NOTE: Much of the following code is heavily inspired by the `helix-loader` crate with license MPL-2.0.
    // To be safe, assume any and all of the following code is MLP-2.0 and copyrighted to the Helix project.
    pub fn grammar(
        &self,
    ) -> Result<topiary_tree_sitter_facade::Language, TopiaryConfigFetchingError> {
        self.grammar_with(&LocalRepos::new())
    }

    /// Same as [`Language::grammar`], but reuses `repos` so a grammar sharing a git repo
    /// with other grammars or queries only triggers one checkout per Topiary run.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn grammar_with(
        &self,
        repos: &LocalRepos,
    ) -> Result<topiary_tree_sitter_facade::Language, TopiaryConfigFetchingError> {
        let library_path = self.library_path()?;

        // Ensure the compile exists
        if !library_path.is_file() {
            match &self.config.grammar.source {
                GrammarSource::Git { git, subdir } => {
                    let checkout = repos.get_or_insert(git)?;
                    GitSource::compile_grammar(
                        &self.name,
                        library_path.clone(),
                        &checkout,
                        subdir.as_deref(),
                    )?;
                }
                GrammarSource::Path { .. } => {
                    return Err(TopiaryConfigFetchingError::GrammarFileNotFound(
                        library_path,
                    ));
                }
            }
        }

        assert!(library_path.is_file());
        log::debug!("Loading grammar from {}", library_path.display());

        use libloading::{Library, Symbol};

        let library = unsafe { Library::new(&library_path) }?;
        let language_fn_name = if let Some(symbol_name) = self.config.grammar.symbol.clone() {
            symbol_name
        } else {
            format!("tree_sitter_{}", self.name.replace('-', "_"))
        };

        let language = unsafe {
            let language_fn: Symbol<unsafe extern "C" fn() -> *const ()> =
                library.get(language_fn_name.as_bytes())?;
            tree_sitter_language::LanguageFn::from_raw(*language_fn)
        };
        std::mem::forget(library);
        Ok(topiary_tree_sitter_facade::Language::from(language))
    }

    #[cfg(target_arch = "wasm32")]
    #[allow(clippy::result_large_err)]
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

type Result<T, E = TopiaryConfigFetchingError> = std::result::Result<T, E>;

trait GitResult<T> {
    fn wrap_err(self) -> Result<T>;
}

impl<T, E: Into<anyhow::Error>> GitResult<T> for Result<T, E> {
    fn wrap_err(self) -> Result<T> {
        self.map_err(|e| TopiaryConfigFetchingError::Git(e.into()))
    }
}

/// A single shallow checkout of a `GitSource` under a `TempDir`, deleted when dropped.
#[derive(Debug)]
pub struct LocalRepo(TempDir);

impl LocalRepo {
    /// Root of the checkout on disk.
    pub fn path(&self) -> &Path {
        self.0.path()
    }
}

impl AsRef<Path> for LocalRepo {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

/// Process-local cache of shallow git checkouts keyed by [`GitSource`], so a single repo
/// hosting multiple grammars or queries is fetched at most once per Topiary run.
#[derive(Debug, Default)]
pub struct LocalRepos {
    // TODO we should eventually omit indexing by rev
    // and just use the normalized git url + git switch <rev>
    repos: Mutex<HashMap<GitSource, LocalRepo>>,
}

impl LocalRepos {
    pub fn new() -> Self {
        Self::default()
    }

    /// fetch on first use
    pub fn get_or_insert(&self, source: &GitSource) -> Result<PathBuf, TopiaryConfigFetchingError> {
        let mut repos = self
            .repos
            .lock()
            .expect("LocalRepos mutex should not be poisoned");
        let repo = match repos.entry(source.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(slot) => slot.insert(source.fetch()?),
        };
        Ok(repo.path().to_path_buf())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl GitSource {
    /// This function is heavily inspired by the one used in Nickel:
    /// <https://github.com/tweag/nickel/blob/master/git/src/lib.rs>
    pub fn fetch(&self) -> Result<LocalRepo, TopiaryConfigFetchingError> {
        let dest = tempfile::tempdir()?;

        // Fetch the git directory somewhere temporary.
        let git_tempdir = tempfile::tempdir().wrap_err()?;
        let repo = gix::init(git_tempdir.path()).wrap_err()?;

        let remote = repo
            .remote_at(self.git.as_str())
            .wrap_err()?
            .with_fetch_tags(fetch::Tags::None)
            .with_refspecs(Some(self.rev.as_str()), Direction::Fetch)
            .wrap_err()?;

        // This does similar credentials stuff to the git CLI (e.g. it looks for ssh
        // keys if it's a fetch over ssh, or it tries to run `askpass` if it needs
        // credentials for https). Maybe we want to have explicit credentials
        // configuration instead of or in addition to the default?
        let connection = remote.connect(Direction::Fetch).wrap_err()?;
        let outcome = connection
            .prepare_fetch(&mut Discard, remote::ref_map::Options::default())
            .wrap_err()?
            // For now, we always fetch shallow. Maybe for the index it's more efficient to
            // keep a single repo around and update it? But that might be in another method.
            .with_shallow(fetch::Shallow::DepthAtRemote(NonZero::new(1).unwrap()))
            .receive(&mut Discard, &IS_INTERRUPTED)
            .wrap_err()?;

        if outcome.ref_map.mappings.len() > 1 {
            return Err(anyhow!("we only asked for 1 ref; why did we get more?")).wrap_err();
        }
        if outcome.ref_map.mappings.is_empty() {
            return Err(anyhow!("Ref not found: {:?} {:?}", self.git, self.rev,)).wrap_err();
        }

        let object_id = source_object_id(&outcome.ref_map.mappings[0].remote)?;
        let object = repo.find_object(object_id).wrap_err()?;
        let tree_id = object.peel_to_tree().wrap_err()?.id();
        let mut index = repo.index_from_tree(&tree_id).wrap_err()?;

        log::info!("Checking out {} {}", self.git, self.rev);
        checkout(
            &mut index,
            dest.path(),
            repo.objects.clone(),
            &Discard,
            &Discard,
            &IS_INTERRUPTED,
            checkout::Options {
                overwrite_existing: true,
                ..Default::default()
            },
        )
        .wrap_err()?;
        index.write(Default::default()).wrap_err()?;

        Ok(LocalRepo(dest))
    }

    /// Compile the tree-sitter grammar rooted at `checkout` + optional `subdir`.
    pub fn compile_grammar(
        name: &str,
        library_path: PathBuf,
        checkout: &Path,
        subdir: Option<&Path>,
    ) -> Result<(), TopiaryConfigFetchingError> {
        let grammar_path = match subdir {
            Some(subdir) => checkout.join(subdir),
            None => checkout.to_path_buf(),
        };

        log::info!("{name}: Building grammar");
        let mut loader =
            tree_sitter_loader::Loader::new().map_err(TopiaryConfigFetchingError::Build)?;
        loader.debug_build(false);
        loader.force_rebuild(true);
        loader
            .compile_parser_at_path(&grammar_path, library_path, &[])
            .map_err(TopiaryConfigFetchingError::Build)?;

        log::info!("{name}: Grammar successfully compiled");
        Ok(())
    }
}

fn source_object_id(source: &refmap::Source) -> Result<ObjectId> {
    match source {
        refmap::Source::ObjectId(id) => Ok(*id),
        refmap::Source::Ref(r) => {
            let (_name, id, peeled) = r.unpack();

            Ok(peeled
                .or(id)
                .ok_or_else(|| anyhow!("unborn reference"))
                .wrap_err()?
                .to_owned())
        }
    }
}
