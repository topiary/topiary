use std::{
    collections::{
        HashMap,
        hash_map::{DefaultHasher, Entry},
    },
    hash::{Hash, Hasher},
    sync::Arc,
};

use tokio::sync::Mutex;
use topiary_core::Language;

use crate::{error::CLIResult, io::InputFile};

/// Thread-safe language definition cache
pub struct LanguageDefinitionCache(Mutex<HashMap<u64, Arc<Language>>>);

impl LanguageDefinitionCache {
    pub fn new() -> Self {
        LanguageDefinitionCache(Mutex::new(HashMap::new()))
    }

    // pub fn fetch_bl

    /// Fetch the language definition from the cache, populating if necessary, with thread-safety
    pub async fn fetch<'i>(&self, input: &'i InputFile<'i>) -> CLIResult<Arc<Language>> {
        // There's no need to store the input's identifying information (language name and query)
        // in the key, so we use its hash directly. This side-steps any awkward lifetime issues.
        let key = {
            let mut hash = DefaultHasher::new();
            input.language().name.hash(&mut hash);
            input.query().hash(&mut hash);

            hash.finish()
        };

        // Lock the entire `HashMap` on access. (This may seem blunt, but is necessary for the
        // correct behaviour when we have near-simultaneous cache access; see issue #605.)
        let mut cache = self.0.lock().await;

        Ok(match cache.entry(key) {
            // Return the language definition from the cache, if it exists...
            Entry::Occupied(lang_def) => {
                log::debug!(
                    "Cache {:p}: Hit at {:#016x} ({}, {})",
                    self,
                    key,
                    input.language().name,
                    input.query()
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
                    input.query()
                );

                let lang_def = Arc::new(input.to_language().await?);
                slot.insert(lang_def).to_owned()
            }
        })
    }
}
