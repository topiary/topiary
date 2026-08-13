//! Topiary can be configured using the `Configuration` struct.
//! A basic configuration, written in Nickel, is included at build time and parsed at runtime.
//! Additional configuration has to be provided by the user of the library.
pub mod error;
pub mod language;
pub mod source;

use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
};

use language::{Language, LanguageConfiguration};
use nickel_lang_core::{
    error::NullReporter,
    eval::{
        cache::CacheImpl,
        value::{NickelValue, lazy::CBNCache},
    },
    position::TermPos,
    program::ProgramBuilder,
};
use serde::Deserialize;

#[cfg(not(target_arch = "wasm32"))]
use crate::error::TopiaryConfigFetchingError;
#[cfg(not(target_arch = "wasm32"))]
use crate::language::LocalRepos;

use crate::error::{TopiaryConfigError, TopiaryConfigResult};

pub use source::Source;

/// The configuration of the Topiary.
///
/// Contains information on how to format every language the user is interested in, modulo what is
/// supported. It can be provided by the user of the library, or alternatively, Topiary ships with
/// default configuration that can be accessed using `Configuration::default`.
#[derive(Debug, Clone)]
pub struct Configuration {
    languages: Vec<Language>,
}

/// Internal struct to help with deserialisation, converted to the actual Configuration in deserialization
#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
struct SerdeConfiguration {
    languages: HashMap<String, LanguageConfiguration>,
}

impl Configuration {
    /// Consume the configuration from the usual sources.
    /// Which sources exactly can be read in the documentation of `Source`.
    ///
    /// # Errors
    ///
    /// If the configuration file does not exist, this function will return a `TopiaryConfigError`
    /// with the path that was not found.
    /// If the configuration file exists, but cannot be parsed, this function will return a
    /// `TopiaryConfigError` with the error that occurred.
    pub fn fetch(merge: bool, file: Option<&Path>) -> TopiaryConfigResult<(Self, Program)> {
        // If we have an explicit file, fail if it doesn't exist
        if let Some(path) = file
            && !path.exists()
        {
            return Err(TopiaryConfigError::FileNotFound(path.to_path_buf()));
        }

        if merge {
            // Get all available configuration sources
            let sources: Vec<Source> = Source::fetch_all(file);

            // And ask Nickel to parse and merge them
            Self::parse(&sources)
        } else {
            // Get the available configuration with best priority
            match Source::fetch_one(file) {
                Source::Builtin => Self::parse(&[Source::Builtin]),
                source => Self::parse(&[source, Source::Builtin]),
            }
        }
    }

    /// Gets a language configuration from the entire configuration.
    ///
    /// # Errors
    ///
    /// If the provided language name cannot be found in the `Configuration`, this
    /// function returns a `TopiaryConfigError`
    pub fn get_language_cfg<T>(&self, name: T) -> TopiaryConfigResult<&Language>
    where
        T: AsRef<str> + fmt::Display,
    {
        self.languages
            .iter()
            .find(|language| language.name == name.as_ref())
            .ok_or(TopiaryConfigError::UnknownLanguage(name.to_string()))
    }

    /// Prefetch a language's grammar and queries per its configuration.
    ///
    /// # Errors
    ///
    /// If any grammar could not build, a `TopiaryConfigFetchingError` is returned.
    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_language(
        language: &Language,
        force: bool,
        repos: &LocalRepos,
    ) -> Result<(), TopiaryConfigFetchingError> {
        match &language.config.grammar.source {
            language::GrammarSource::Git { git, subdir } => {
                let library_path = language.library_path()?;

                log::info!(
                    "Fetch \"{}\": Configured via Git ({} ({})); to {}",
                    language.name,
                    git.git,
                    git.rev,
                    library_path.display()
                );

                if !force && library_path.is_file() {
                    log::info!(
                        "{}: Built grammar already exists; nothing to do",
                        language.name
                    );
                } else {
                    let checkout = repos.get_or_insert(git)?;
                    language::GitSource::compile_grammar(
                        &language.name,
                        library_path,
                        &checkout,
                        subdir.as_deref(),
                    )?;
                }
            }

            language::GrammarSource::Path(path) => {
                log::info!(
                    "Fetch \"{}\": Configured via filesystem ({}); nothing to do",
                    language.name,
                    path.display(),
                );

                if !path.exists() {
                    return Err(TopiaryConfigFetchingError::GrammarFileNotFound(
                        path.to_path_buf(),
                    ));
                }
            }
        }

        // Ensure `topiary prefetch` covers both grammars and queries.
        if let Some(queries) = language.config.queries.as_ref() {
            for (query_name, query) in queries {
                if query.source.git.is_none() {
                    continue;
                }
                log::info!(
                    "Fetch \"{}\": prefetching {query_name} query",
                    language.name,
                );
                language.resolve_query_path_with(&query.source, repos)?;
            }
        }

        Ok(())
    }

    /// Prefetches and builds the desired language.
    /// This can be beneficial to speed up future startup time.
    ///
    /// # Errors
    ///
    /// If the language could not be found or the Grammar could not be build, a `TopiaryConfigError` is returned.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn prefetch_language<T>(&self, language: T, force: bool) -> TopiaryConfigResult<()>
    where
        T: AsRef<str> + fmt::Display,
    {
        let repos = LocalRepos::new();
        let l = self.get_language_cfg(language)?;
        Configuration::fetch_language(l, force, &repos)?;
        Ok(())
    }

    /// Prefetches and builds all known languages.
    /// This can be beneficial to speed up future startup time.
    ///
    /// # Errors
    ///
    /// If any Grammar could not be build, a `TopiaryConfigError` is returned.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn prefetch_languages(&self, force: bool) -> TopiaryConfigResult<()> {
        let repos = LocalRepos::new();

        // When the `parallel` feature is enabled (which it is by default), we use Rayon to fetch
        // and compile all found grammars concurrently.
        // NOTE The MSVC linker does not seem to like concurrent builds, so concurrency is disabled
        // on Windows (see https://github.com/topiary/topiary/issues/868)
        #[cfg(all(feature = "parallel", not(windows)))]
        {
            use rayon::prelude::*;
            self.languages
                .par_iter()
                .map(|l| Configuration::fetch_language(l, force, &repos))
                .collect::<Result<Vec<_>, TopiaryConfigFetchingError>>()?;
        }

        #[cfg(any(not(feature = "parallel"), windows))]
        {
            self.languages
                .iter()
                .map(|l| Configuration::fetch_language(l, force, &repos))
                .collect::<Result<Vec<_>, TopiaryConfigFetchingError>>()?;
        }

        Ok(())
    }

    /// Convenience alias to detect the Language from a Path-like value's extension.
    ///
    /// # Errors
    ///
    /// If the file extension is not supported, a `FormatterError` will be returned.
    pub fn detect<P: AsRef<Path>>(&self, path: P) -> TopiaryConfigResult<&Language> {
        let pb = &path.as_ref().to_path_buf();
        if let Some(extension) = pb.extension().and_then(|ext| ext.to_str()) {
            for lang in &self.languages {
                if lang.config.extensions.contains(extension) {
                    return Ok(lang);
                }
            }
            return Err(TopiaryConfigError::UnknownExtension(extension.to_string()));
        }
        Err(TopiaryConfigError::NoExtension(pb.clone()))
    }

    fn parse(sources: &[Source]) -> TopiaryConfigResult<(Self, Program)> {
        let mut program = Program::build_with_sources(sources)?;
        let ncl = program.eval_full_for_export()?;

        let serde_config = SerdeConfiguration::deserialize(ncl).map_err(|error| {
            TopiaryConfigError::NickelDeserialization {
                error,
                files: Box::new(program.files()),
            }
        })?;

        Ok((serde_config.into(), program))
    }
}

impl Default for Configuration {
    /// Return the built-in configuration
    // This is particularly useful for testing
    fn default() -> Self {
        let mut program = Program::build_with_sources(&[Source::Builtin])
            .expect("Evaluating the builtin configuration should be safe");
        let ncl = program
            .eval_full_for_export()
            .expect("Evaluating the builtin configuration should be safe");
        let serde_config = SerdeConfiguration::deserialize(ncl)
            .expect("Evaluating the builtin configuration should be safe");

        serde_config.into()
    }
}

/// Convert `Serialisation` values into `HashMap`s, keyed on `Language::name`
impl From<&Configuration> for HashMap<String, Language> {
    fn from(config: &Configuration) -> Self {
        HashMap::from_iter(
            config
                .languages
                .iter()
                .map(|language| (language.name.clone(), language.clone())),
        )
    }
}

// Order-invariant equality; required for unit testing
impl PartialEq for Configuration {
    fn eq(&self, other: &Self) -> bool {
        let lhs: HashMap<String, Language> = self.into();
        let rhs: HashMap<String, Language> = other.into();

        lhs == rhs
    }
}

impl From<SerdeConfiguration> for Configuration {
    fn from(value: SerdeConfiguration) -> Self {
        let languages = value
            .languages
            .into_iter()
            .map(|(name, config)| Language::new(name, config))
            .collect();

        Self { languages }
    }
}

pub(crate) fn project_dirs() -> directories::ProjectDirs {
    directories::ProjectDirs::from("", "", "topiary")
        .expect("Could not access the OS's Home directory")
}

pub struct Program {
    inner: nickel_lang_core::program::Program<CBNCache>,
}

impl From<nickel_lang_core::program::Program<CBNCache>> for Program {
    fn from(program: nickel_lang_core::program::Program<CBNCache>) -> Self {
        Self { inner: program }
    }
}

impl Program {
    fn builder() -> ProgramBuilder<NullReporter, std::io::Stderr> {
        ProgramBuilder::new()
            .with_trace(std::io::stderr())
            .with_reporter(NullReporter {})
    }

    pub fn build() -> TopiaryConfigResult<Self> {
        Ok(Self::builder().build::<CacheImpl>()?.into())
    }

    pub fn eval_full_for_export(&mut self) -> TopiaryConfigResult<NickelValue> {
        let ncl = self
            .inner
            .eval_full_for_export()
            .map_err(|error| TopiaryConfigError::nickel(error, self.files()))?;
        Ok(ncl)
    }

    /// Evaluate `field_path` using [`Program::parse_field_path`]
    pub fn query_field(&mut self, field_path: &str) -> TopiaryConfigResult<NickelValue> {
        let mut field = self
            .parse_field_path(field_path.to_owned())
            .map_err(|e| TopiaryConfigError::nickel(e.into(), self.files()))?;
        std::mem::swap(&mut self.inner.field, &mut field);

        let ncl = self.eval_full_for_export()?;

        // replace with previous field path
        std::mem::swap(&mut self.inner.field, &mut field);

        Ok(ncl)
    }

    fn build_with_sources(sources: &[Source]) -> TopiaryConfigResult<Self> {
        let mut builder = Self::builder();
        for source in sources {
            builder = source.clone().add_to(builder);
        }
        let program = builder.build::<CacheImpl>()?;
        Ok(program.into())
    }

    pub fn get_source(&mut self, ncl: &NickelValue) -> TopiaryConfigResult<Option<PathBuf>> {
        let pos = self.inner.pos_table().get(ncl.pos_idx());
        let id = match pos {
            TermPos::Original(s) | TermPos::Inherited(s) => s.src_id,
            TermPos::None => return Ok(None),
        };
        let files = self.files();
        let name = files.name(id);
        Ok(Some(PathBuf::from(name)))
        // self.custom_transform(0, |cache, table, ncl| {
        //     let pos = table.get(ncl.pos_idx());
        // });
        // let pos_idx = ncl.pos_idx();
        // let vm = self.new_vm();
    }
}

impl std::ops::Deref for Program {
    type Target = nickel_lang_core::program::Program<CBNCache>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Program {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
