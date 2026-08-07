use std::{
    collections::{
        HashMap,
        hash_map::{DefaultHasher, Entry},
    },
    hash::{Hash, Hasher},
    sync::{Arc, Mutex, OnceLock},
};

use crate::Configuration;
use topiary_config::language::GrammarLoader;
use topiary_core::Language;

use crate::error::{CLIResult, ResultPreformat};
use crate::io::InputFile;

/// Thread-safe language definition cache
#[derive(Debug)]
pub struct LanguageDefinitionCache {
    languages: Mutex<HashMap<u64, Arc<Language>>>,
    loader: OnceLock<GrammarLoader>,
}

impl LanguageDefinitionCache {
    pub fn new() -> Self {
        LanguageDefinitionCache {
            languages: Mutex::new(HashMap::new()),
            loader: OnceLock::new(),
        }
    }

    pub fn loader(
        &self,
    ) -> Result<&GrammarLoader, topiary_config::error::TopiaryConfigFetchingError> {
        if let Some(loader) = self.loader.get() {
            return Ok(loader);
        }
        let loader = GrammarLoader::for_project("topiary")?;
        let _ = self.loader.set(loader);
        Ok(self
            .loader
            .get()
            .expect("a loader was initialized by this or another thread"))
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

    /// Fetch the language definition from the cache, populating if necessary, with thread-safety
    pub fn fetch_input<'i>(&self, input: &'i InputFile<'i>) -> CLIResult<Arc<Language>> {
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

                let loader = self
                    .loader()
                    .map_err(topiary_config::error::TopiaryConfigError::from)
                    .preformat_context()?;
                let lang_def = Arc::new(input.to_language_sync(loader)?);
                slot.insert(lang_def).to_owned()
            }
        })
    }

    /// Fetch an injected language definition by name from the same cache used for input languages.
    pub fn fetch_from_config(
        &self,
        config: &Configuration,
        name: &str,
    ) -> CLIResult<Arc<Language>> {
        let formatting_query = config.get_query_source(name, topiary_queries::FORMATTING_QUERY)?;
        let injection_query = config
            .get_query_source(name, topiary_queries::INJECTIONS_QUERY)
            .ok();
        let key = Self::key_for_parts(name, &formatting_query, injection_query.as_ref());

        let mut cache = self
            .languages
            .lock()
            .expect("language cache mutex poisoned");

        Ok(match cache.entry(key) {
            Entry::Occupied(lang_def) => {
                log::debug!("Cache {:p}: Hit at {:#016x} ({name})", self, key);
                lang_def.get().to_owned()
            }

            Entry::Vacant(slot) => {
                log::debug!("Cache {:p}: Insert at {:#016x} ({name})", self, key);
                let lang_def = Arc::new(config.get_language(name)?);
                slot.insert(lang_def).to_owned()
            }
        })
    }
}
