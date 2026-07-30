use std::{
    collections::{
        HashMap,
        hash_map::{DefaultHasher, Entry},
    },
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use crate::Configuration;
use topiary_config::language::LocalRepos;
use topiary_core::Language;
use topiary_queries;

use crate::error::CLIResult;
use crate::io::InputFile;

/// Thread-safe language definition cache
#[derive(Debug)]
pub struct LanguageDefinitionCache {
    languages: Mutex<HashMap<u64, Arc<Language>>>,
    repos: LocalRepos,
}

impl LanguageDefinitionCache {
    pub fn new() -> Self {
        LanguageDefinitionCache {
            languages: Mutex::new(HashMap::new()),
            repos: LocalRepos::new(),
        }
    }

    pub fn repos(&self) -> &LocalRepos {
        &self.repos
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

                let lang_def = Arc::new(input.to_language_sync()?);
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
