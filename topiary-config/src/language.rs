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
use std::collections::HashSet;
#[cfg(not(target_arch = "wasm32"))]
use std::num::NonZero;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

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
    #[serde(rename = "git")]
    Git(GitSource),
    #[serde(rename = "path")]
    Path(PathBuf),
}

#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct GitSource {
    /// The URL of the git repository that contains the tree-sitter grammar.
    pub git: String,
    /// The revision of the git repository to use.
    pub rev: String,
    /// The sub-directory within the repository where the grammar is located. Defaults to the root of the repository
    pub subdir: Option<String>,
}

impl Language {
    pub fn new(name: String, config: LanguageConfiguration) -> Self {
        Self { name, config }
    }

    pub fn indent(&self) -> Option<String> {
        self.config.indent.clone()
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::result_large_err)]
    pub fn find_query_file(&self) -> TopiaryConfigResult<PathBuf> {
        use crate::source::Source;

        let name = self.name.as_str();

        #[rustfmt::skip]
        let potentials: [Option<PathBuf>; 5] = [
            std::env::var("TOPIARY_LANGUAGE_DIR").map(PathBuf::from).ok(),
            option_env!("TOPIARY_LANGUAGE_DIR").map(PathBuf::from),
            Source::fetch_one(&None).queries_dir(),
            Some(PathBuf::from("./topiary-queries/queries")),
            Some(PathBuf::from("../topiary-queries/queries")),
        ];

        potentials
            .into_iter()
            .flatten()
            .flat_map(|path| {
                [
                    // New layout: <dir>/<lang>/formatting.scm
                    path.join(name).join(topiary_queries::FORMATTING_QUERY),
                    // Old layout: <dir>/<lang>.scm
                    path.join(format!("{name}.scm")),
                ]
            })
            .find(|path| path.exists())
            .ok_or_else(|| TopiaryConfigError::QueryFileNotFound(PathBuf::from(name)))
    }

    #[cfg(not(target_arch = "wasm32"))]
    // Returns the library path, and ensures the parent directories exist.
    pub fn library_path(&self) -> std::io::Result<PathBuf> {
        match &self.config.grammar.source {
            GrammarSource::Git(git_source) => {
                let mut library_path = crate::project_dirs().cache_dir().to_path_buf();
                library_path.push(self.name.clone());
                std::fs::create_dir_all(&library_path)?;

                // Set the output path as the revision of the grammar,
                // with a platform-appropriate extension
                library_path.push(git_source.rev.clone());
                library_path.set_extension(std::env::consts::DLL_EXTENSION);

                Ok(library_path)
            }

            GrammarSource::Path(path) => Ok(path.to_path_buf()),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    // NOTE: Much of the following code is heavily inspired by the `helix-loader` crate with license MPL-2.0.
    // To be safe, assume any and all of the following code is MLP-2.0 and copyrighted to the Helix project.
    pub fn grammar(
        &self,
    ) -> Result<topiary_tree_sitter_facade::Language, TopiaryConfigFetchingError> {
        let library_path = self.library_path()?;

        // Ensure the compile exists
        if !library_path.is_file() {
            match &self.config.grammar.source {
                GrammarSource::Git(git_source) => {
                    git_source.fetch_and_compile(&self.name, library_path.clone())?
                }
                GrammarSource::Path(_) => {
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

#[cfg(not(target_arch = "wasm32"))]
impl GitSource {
    fn fetch_and_compile(
        &self,
        name: &str,
        library_path: PathBuf,
    ) -> Result<(), TopiaryConfigFetchingError> {
        log::info!("{name}: Language Grammar not found, attempting to fetch and compile it");
        // Create a temporary directory to clone the repository to. We could
        // cached the repositories, but the additional disk space is probably
        // not worth the benefits gained by caching. The tempdir is deleted
        // when dropped
        let tmp_dir = tempfile::tempdir()?;

        self.fetch_and_compile_with_dir(name, library_path, false, tmp_dir.keep())
    }

    /// This function is heavily inspired by the one used in Nickel:
    /// https://github.com/tweag/nickel/blob/master/git/src/lib.rs
    pub fn fetch_and_compile_with_dir(
        &self,
        name: &str,
        library_path: PathBuf,
        force: bool,
        tmp_dir: PathBuf,
    ) -> Result<(), TopiaryConfigFetchingError> {
        if !force && library_path.is_file() {
            log::info!("{name}: Built grammar already exists; nothing to do");
            return Ok(());
        }
        let tmp_dir = tmp_dir.join(name);
        std::fs::create_dir_all(&tmp_dir)?;

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

        log::info!("{}: Checking out {} {}", name, self.git, self.rev);
        checkout(
            &mut index,
            &tmp_dir,
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

        // Update the build path for grammars that are not defined at the repo root
        let grammar_path = match self.subdir.clone() {
            // Some grammars are in a subdirectory, go there
            Some(subdir) => tmp_dir.join(subdir),
            None => tmp_dir,
        };

        // Build grammar
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
