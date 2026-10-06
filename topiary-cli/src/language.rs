use std::{
    collections::{
        HashMap, HashSet,
        hash_map::{DefaultHasher, Entry},
    },
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use rootcause::report;

use crate::Configuration;
use topiary_config::language::LocalRepos;
use topiary_core::Language;

use crate::error::{CLIResult, TopiaryError};
use crate::io::InputFile;

/// Thread-safe language definition cache
#[derive(Debug)]
pub struct LanguageDefinitionCache {
    languages: Mutex<HashMap<u64, Arc<Language>>>,
    repos: LocalRepos,
    /// Languages excluded via `--skip-language`.
    ///
    /// Skipping only affects *injected* languages (and the associated query
    /// searching); formatting an input whose own language is skipped is an
    /// error, as the host grammar is required to format the file.
    skip_languages: HashSet<String>,
}

impl LanguageDefinitionCache {
    pub fn new(skipped: impl IntoIterator<Item = String>) -> Self {
        LanguageDefinitionCache {
            languages: Mutex::new(HashMap::new()),
            repos: LocalRepos::new(),
            skip_languages: skipped.into_iter().collect(),
        }
    }

    pub fn repos(&self) -> &LocalRepos {
        &self.repos
    }

    /// Returns `true` if `name` is excluded via `--skip-language`.
    pub fn is_skipped(&self, name: &str) -> bool {
        self.skip_languages.contains(name)
    }

    fn key_for_parts(
        language_name: &str,
        formatting_query: &impl Hash,
        injection_query: Option<&impl Hash>,
    ) -> u64 {
        let mut hash = DefaultHasher::new();
        language_name.hash(&mut hash);
        formatting_query.hash(&mut hash);
        injection_query.hash(&mut hash);

        hash.finish()
    }

    /// Fetch the language definition from the cache, populating if necessary, with thread-safety.
    ///
    /// This is the host-language path: a language excluded via `--skip-language` is an
    /// error, since the host grammar is required to format the input.
    pub fn fetch_input<'i>(&self, input: &'i InputFile<'i>) -> CLIResult<Arc<Language>> {
        let name = &input.language().name;
        if self.is_skipped(name) {
            return Err(report!(TopiaryError::SkippedHostLanguage(name.clone())).into_dynamic());
        }

        // There's no need to store the input's identifying information (language name and query)
        // in the key, so we use its hash directly. This side-steps any awkward lifetime issues.
        let key = Self::key_for_parts(
            &input.language().name,
            input.formatting_query(),
            input.injection_query(),
        );

        // Lock the entire `HashMap` on access. (This may seem blunt, but is necessary for the
        // correct behaviour when we have near-simultaneous cache access; see issue #605.)
        let mut cache = self
            .languages
            .lock()
            .expect("language cache mutex poisoned");

        Ok(match cache.entry(key) {
            // Return the language definition from the cache, if it exists...
            Entry::Occupied(lang_def) => {
                log::debug!(
                    "Cache {:p}: Hit at {:#016x} ({}, {})",
                    self,
                    key,
                    input.language().name,
                    input.formatting_query()
                );

                lang_def.get().to_owned()
            }

            // ...otherwise, fetch the language definition, to populate the cache
            Entry::Vacant(slot) => {
                log::debug!(
                    "Cache {:p}: Insert at {:#016x} ({}, {})",
                    self,
                    key,
                    input.language().name,
                    input.formatting_query()
                );

                let lang_def = Arc::new(input.to_language_sync(self.repos())?);
                slot.insert(lang_def).to_owned()
            }
        })
    }

    /// Fetch an injected language definition by name from the same cache used for input languages.
    pub fn fetch_from_config(
        &self,
        config: &Configuration,
        language: &str,
    ) -> CLIResult<Arc<Language>> {
        let formatting_query =
            config.get_query_source(language, topiary_queries::FORMATTING_QUERY)?;
        let injection_query = config
            .get_query_source(language, topiary_queries::INJECTIONS_QUERY)
            .ok();
        let key = Self::key_for_parts(language, &formatting_query, injection_query.as_ref());

        let mut cache = self
            .languages
            .lock()
            .expect("language cache mutex poisoned");

        Ok(match cache.entry(key) {
            Entry::Occupied(lang_def) => {
                log::debug!("Cache {self:p}: Hit at {key:#016x} ({language})");
                lang_def.get().to_owned()
            }

            Entry::Vacant(slot) => {
                log::debug!("Cache {self:p}: Insert at {key:#016x} ({language})");
                let lang_def = Arc::new(config.get_language(language)?);
                slot.insert(lang_def).to_owned()
            }
        })
    }

    /// Fetch _injected_ language by name, filtering out any language excluded languages.
    ///
    /// NOTE: Unlike [`Self::fetch_input`], failing to fetch a skipped language should not result in
    /// an error so long as the language is for an injected grammar; such a `rust` code fence inside
    /// a markdown document.
    pub fn fetch_injected(
        &self,
        config: &Configuration,
        language: &str,
    ) -> CLIResult<Option<Arc<Language>>> {
        if self.is_skipped(language) {
            log::debug!("Skipping injected language: {language}");
            return Ok(None);
        }

        self.fetch_from_config(config, language).map(Some)
    }
}
