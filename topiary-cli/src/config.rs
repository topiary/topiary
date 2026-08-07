use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::{ops::Deref, path::Path};

use nickel_lang_core::eval::value::NickelValue;
use rootcause::prelude::ResultExt;
use topiary_config::source::Source;
use topiary_core::{
    FormatterError, FormatterResult, InjectionQuery, Language, SpanAttachment, TopiaryQuery,
};

use crate::error::{CLIResult, ResultPreformat, TopiaryError};
use crate::language::LanguageDefinitionCache;

thread_local! {
    static NICKEL_VALUES: Cell<Vec<Rc<NickelValue>>> = const { Cell::new(Vec::new()) };
    static NEXT_ID: Cell<u32> = const { Cell::new(0) };
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
            vec.push(Rc::new(ncl));
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
    /// Returns an [`Rc<NickelValue>`] which allows cheap cloning via reference counting.
    /// The reference is guaranteed to be valid because we increment the counter on every
    /// `Configuration::new()` call, ensuring the index is always within bounds of the thread-local storage.
    pub fn ncl(&self) -> Rc<NickelValue> {
        NICKEL_VALUES.with(|values| {
            // `Cell` does not give out mutable references (unsafe in thread-local context);
            // instead one has to take a value, modify/use it, and put it back.
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
    pub fn iter_sources(&self) -> impl Iterator<Item = (&'static str, Source)> {
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
        let name_ref = name.as_ref();
        let config_language = self.get_language_cfg(name_ref).preformat_context()?;
        let loader = self
            .cache
            .loader()
            .map_err(topiary_config::error::TopiaryConfigError::from)
            .preformat_context()?;
        let grammar = config_language.grammar_with(loader)?;
        let query_source = self.get_query_source(name_ref, topiary_queries::FORMATTING_QUERY)?;
        let query_content = query_source.get_content_sync()?;
        let formatting_query = TopiaryQuery::new(&grammar, &query_content)
            .attach_filepath(query_source.filepath())
            .context(FormatterError::Parsing)?;
        let injection_query = match self
            .get_query_source(name_ref, topiary_queries::INJECTIONS_QUERY)
        {
            Ok(source) => {
                let contents = source.get_content_sync()?;
                Some(InjectionQuery::new(&grammar, &contents).attach_filepath(source.filepath())?)
            }
            Err(_) => None,
        };
        Ok(topiary_core::Language {
            name: name_ref.to_string(),
            formatting_query,
            injection_query,
            grammar,
            indent: config_language.indent(),
        })
    }

    /// Get a query source for the given language and query name, using the cached repos
    pub fn get_query_source(
        &self,
        language_name: &str,
        query_name: &str,
    ) -> CLIResult<crate::io::QuerySource> {
        let config_language = self.get_language_cfg(language_name).preformat_context()?;
        let cache = self.cache();
        let loader = cache
            .loader()
            .map_err(topiary_config::error::TopiaryConfigError::from)
            .preformat_context()?;
        let find = config_language.find_query_file_with(query_name, loader);
        let query: crate::io::QuerySource = match find {
            Ok(p) => p.into(),
            // For some reason, Topiary could not find any
            // matching file in a default location. As a final attempt, try the
            // builtin ones. Store the error, return that if we
            // fail to find anything, because the builtin error might be unexpected.
            Err(e) => {
                log::warn!(
                    "No {query_name} query files found in any of the expected locations. Falling back to compile-time included files."
                );
                query_from_builtin(&config_language.name, query_name)
                    .local_context(e)
                    .preformat_context()?
            }
        };
        Ok(query)
    }

    pub(crate) fn cache(&self) -> Arc<LanguageDefinitionCache> {
        self.cache.clone()
    }

    /// Resolve an injected language by name, returning None if the language is unknown.
    ///
    /// This is used to fetch language definitions for code injections during formatting.
    /// Returns `Ok(None)` if the language is not configured, `Ok(Some(language))` if found,
    /// or `Err` if there was an error resolving the language.
    pub fn resolve_injected_language(&self, name: &str) -> FormatterResult<Option<Arc<Language>>> {
        if matches!(
            self.get_language_cfg(name),
            Err(topiary_config::error::TopiaryConfigError::UnknownLanguage(
                _
            ))
        ) {
            return Ok(None);
        }

        match self.cache().fetch_from_config(self, name) {
            Ok(language) => Ok(Some(language)),
            Err(report) => Err(report.context(FormatterError::InjectionLanguageResolution {
                language: name.to_owned(),
            })),
        }
    }
}

/// Get a builtin query for the given language and query name
fn query_from_builtin<T, Q>(language: T, query: Q) -> CLIResult<crate::io::QuerySource>
where
    T: AsRef<str> + std::fmt::Display,
    Q: AsRef<str>,
{
    let name_str = language.as_ref();
    match query.as_ref() {
        topiary_queries::FORMATTING_QUERY => match name_str {
            #[cfg(feature = "bash")]
            "bash" => Ok(topiary_queries::bash().into()),

            #[cfg(feature = "css")]
            "css" => Ok(topiary_queries::css().into()),

            #[cfg(feature = "json")]
            "json" => Ok(topiary_queries::json().into()),

            #[cfg(feature = "markdown")]
            "markdown" => Ok(topiary_queries::markdown().into()),

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

            _ => Err(TopiaryError::UnsupportedLanguage(name_str.to_string()).into()),
        },
        topiary_queries::INJECTIONS_QUERY => match name_str {
            #[cfg(feature = "markdown")]
            "markdown" => Ok(topiary_queries::markdown_injections().into()),

            #[cfg(feature = "menhir")]
            "menhir" => Ok(topiary_queries::menhir_injections().into()),

            #[cfg(feature = "ocamllex")]
            "ocamllex" => Ok(topiary_queries::ocamllex_injections().into()),

            #[cfg(feature = "rust")]
            "rust" => Ok(topiary_queries::rust_injections().into()),

            _ => Err(TopiaryError::UnsupportedLanguage(name_str.to_string()).into()),
        },
        _ => Err(TopiaryError::UnsupportedLanguage(name_str.to_string()).into()),
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

#[cfg(feature = "fancy-config")]
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
        if let Err(err) = formatter(
            &mut nickel_config.as_bytes(),
            &mut output,
            &language,
            Operation::Format {
                skip_idempotence: true,
                tolerate_parsing_errors: false,
            },
            None,
        ) {
            log::error!(
                "error calling {}::fmt : {err}",
                std::any::type_name::<Self>()
            );
            return write!(f, "{}", self.ncl());
        }

        write!(f, "{}", String::from_utf8_lossy(&output))
    }
}

#[cfg(not(feature = "fancy-config"))]
impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ncl())
    }
}

// Strip field metadata (doc strings, type/contract annotations, `| default`,
// `| optional`, priority) and unwrap `Term::Annotated` nodes from a NickelValue
// so that the pretty printer emits a plain data record.
#[cfg(feature = "fancy-config")]
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
