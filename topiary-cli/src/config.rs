use std::cell::Cell;
use std::path::PathBuf;
use std::sync::Arc;
use std::{ops::Deref, path::Path};

use nickel_lang_core::eval::value::NickelValue;
use rootcause::prelude::ResultExt;
use topiary_config::language::LocalRepos;
use topiary_config::source::Source;
use topiary_core::{FormatterError, InjectionQuery, SpanAttachment, TopiaryQuery};

use crate::error::{CLIResult, ResultPreformat};
use crate::io::to_query_from_language;
use crate::language::LanguageDefinitionCache;

thread_local! {
    static NICKEL_VALUES: Cell<Vec<Arc<NickelValue>>> = Cell::new(Vec::new());
    static NEXT_ID: Cell<u32> = Cell::new(0);
}

/// Wrapper around Configuration and its raw Nickel representation.
///
/// [`NickelValue`] is used for operations that require the using the Nickel AST,
/// such as formatting or querying the configuration with Nickel-aware tooling.
///
/// The NickelValue is stored in thread-local storage to avoid Send+Sync issues.
#[derive(Debug, Clone)]
pub struct Configuration {
    inner: topiary_config::Configuration,
    ncl_id: u32,
    path: Option<PathBuf>,
    cache: Arc<LanguageDefinitionCache>,
}

impl Configuration {
    /// Create a new Configuration by fetching from the given path
    pub fn new(merge: bool, path: Option<&Path>) -> CLIResult<Self> {
        let (inner, ncl) = topiary_config::Configuration::fetch(merge, path).preformat_context()?;

        // Store the NickelValue in thread-local storage
        let ncl_id = NEXT_ID.with(|id| {
            let current = id.get();
            id.set(current + 1);
            current
        });

        NICKEL_VALUES.with(|values| {
            let mut vec = values.take();
            vec.push(Arc::new(ncl));
            values.set(vec);
        });

        Ok(Self {
            inner,
            ncl_id,
            path: path.map(|p| p.to_owned()),
            cache: Arc::new(LanguageDefinitionCache::new()),
        })
    }

    /// Get the Nickel value for this configuration
    ///
    /// Returns an [`Arc<NickelValue>`] which allows cheap cloning via reference counting.
    /// The reference is guaranteed to be valid because we increment the counter on every
    /// `Configuration::new()` call, ensuring the index is always within bounds of the thread-local storage.
    pub fn ncl(&self) -> Arc<NickelValue> {
        NICKEL_VALUES.with(|values| {
            let vec = values.take();
            // SAFETY: We have a guarantee that the index is valid because:
            // 1. Each `Configuration` stores an `ncl_id` from `NEXT_ID`
            // 2. Each `Configuration::new()` increments `NEXT_ID` after pushing to `NICKEL_VALUES`
            // 3. We never remove items from `NICKEL_VALUES,` only add
            // Therefore, the index is always valid.
            let result = vec.get(self.ncl_id as usize).unwrap().clone();
            values.set(vec);
            result
        })
    }

    /// Get the config sources
    pub fn config_sources(&self) -> impl Iterator<Item = (&'static str, Source)> {
        Source::config_sources(self.path.as_deref())
    }

    /// Extract a field from the configuration
    pub fn extract_field(&self, merge: bool, field_path: &str) -> CLIResult<NickelValue> {
        let ncl = topiary_config::Configuration::extract_field(merge, &self.path, field_path)
            .preformat_context()?;

        Ok(ncl)
    }

    /// Prefetch a language's grammar and queries
    pub fn prefetch_language<T>(&self, language: T, force: bool) -> CLIResult<()>
    where
        T: AsRef<str> + std::fmt::Display,
    {
        self.inner
            .prefetch_language(language, force)
            .preformat_context()?;

        Ok(())
    }

    /// Prefetch all languages' grammars and queries
    pub fn prefetch_languages(&self, force: bool) -> CLIResult<()> {
        self.inner.prefetch_languages(force).preformat_context()?;

        Ok(())
    }

    /// Build a [`topiary_core::Language`]
    pub fn get_language<T>(&self, name: T) -> CLIResult<topiary_core::Language>
    where
        T: AsRef<str> + std::fmt::Display,
    {
        let config_language = self.get_language_cfg(name.as_ref()).preformat_context()?;
        let grammar = config_language.grammar()?;
        let repos = LocalRepos::new();
        let query_source = to_query_from_language(
            config_language,
            topiary_queries::FORMATTING_QUERY,
            Some(&repos),
        )?;
        let query_content = query_source.get_content_sync()?;
        let formatting_query = TopiaryQuery::new(&grammar, &query_content)
            .attach_filepath(query_source.filepath())
            .context(FormatterError::Parsing)?;
        let injection_query = match to_query_from_language(
            config_language,
            topiary_queries::INJECTIONS_QUERY,
            Some(&repos),
        )
        .ok()
        {
            Some(source) => {
                let contents = source.get_content_sync()?;
                Some(InjectionQuery::new(&grammar, &contents).attach_filepath(source.filepath())?)
            }
            None => None,
        };
        Ok(topiary_core::Language {
            name: name.as_ref().to_string(),
            formatting_query,
            injection_query,
            grammar,
            indent: config_language.indent(),
        })
    }

    pub(crate) fn cache(&self) -> Arc<LanguageDefinitionCache> {
        self.cache.clone()
    }
}

impl Deref for Configuration {
    type Target = topiary_config::Configuration;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<topiary_config::Configuration> for Configuration {
    fn as_ref(&self) -> &topiary_config::Configuration {
        &self.inner
    }
}

#[cfg(feature = "nickel")]
impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use topiary_core::{Operation, formatter};

        // TODO handle verbose flag
        let stripped = strip_metadata((*self.ncl()).clone());
        let nickel_config = format!("{stripped}");

        // if errors are encountered in formatting, return
        let language = match self.get_language("nickel") {
            Ok(lang) => lang,
            Err(_) => return write!(f, "{}", self.ncl()),
        };

        let mut output = Vec::new();
        if let Err(_) = formatter(
            &mut nickel_config.as_bytes(),
            &mut output,
            &language,
            Operation::Format {
                skip_idempotence: true,
                tolerate_parsing_errors: false,
            },
            None,
        ) {
            return write!(f, "{}", self.ncl());
        }

        write!(f, "{}", String::from_utf8_lossy(&output))
    }
}

#[cfg(not(feature = "nickel"))]
impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ncl())
    }
}

// Strip field metadata (doc strings, type/contract annotations, `| default`,
// `| optional`, priority) and unwrap `Term::Annotated` nodes from a NickelValue
// so that the pretty printer emits a plain data record.
#[cfg(feature = "nickel")]
fn strip_metadata(value: NickelValue) -> NickelValue {
    use nickel_lang_core::eval::value::{RecordData, ValueContent};
    use nickel_lang_core::term::Term;
    use nickel_lang_core::traverse::{Traverse, TraverseOrder};

    value
        .traverse(
            &mut |v: NickelValue| -> std::result::Result<NickelValue, std::convert::Infallible> {
                let pos_idx = v.pos_idx();
                match v.content() {
                    ValueContent::Record(lens) => {
                        let Some(record) = lens.take().into_opt() else {
                            return Ok(NickelValue::record_posless(RecordData::empty())
                                .with_pos_idx(pos_idx));
                        };
                        let fields = record
                            .fields
                            .into_iter()
                            .map(|(id, field)| {
                                let nickel_lang_core::term::record::Field { value, .. } = field;
                                (
                                    id,
                                    nickel_lang_core::term::record::Field::from(
                                        value.unwrap_or_else(NickelValue::null),
                                    ),
                                )
                            })
                            .collect();
                        Ok(NickelValue::record(
                            RecordData::new_shared_tail(fields, record.attrs, record.sealed_tail),
                            pos_idx,
                        ))
                    }
                    ValueContent::Term(lens) => {
                        let term = lens.take();
                        if let Term::Annotated(data) = term {
                            Ok(data.inner.clone())
                        } else {
                            Ok(NickelValue::term(term, pos_idx))
                        }
                    }
                    other => Ok(other.restore()),
                }
            },
            TraverseOrder::BottomUp,
        )
        .unwrap_or_else(|never: std::convert::Infallible| match never {})
}
