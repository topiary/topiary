//! A general code formatter that relies on
//! [Tree-sitter](https://tree-sitter.github.io/tree-sitter/) for language
//! parsing.
//!
//! In order for a language to be supported, there must be a [Tree-sitter
//! grammar](https://tree-sitter.github.io/tree-sitter/#available-parsers)
//! available, and there must be a query file that dictates how that language is
//! to be formatted. We include query files for some languages.
//!
//! More details can be found on
//! [GitHub](https://github.com/topiary/topiary).

use std::{io, sync::Arc};

use pretty_assertions::StrComparison;
use rootcause::{prelude::ResultExt, report};
use tree_sitter::Position;

pub use crate::{
    error::{ErrorSpan, FormatterError, SpanAttachment},
    language::Language,
    tree_sitter::{
        CoverageData, InjectionQuery, InjectionSpan, SyntaxNode, TopiaryQuery, Visualisation,
        apply_query, check_query_coverage, collect_injections, parse,
    },
};

mod atom_collection;
mod error;
mod graphviz;
mod language;
mod pretty;
mod tree_sitter;

#[doc(hidden)]
pub mod test_utils;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeInformation {
    line_number: u32,
    scope_id: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Capitalisation {
    UpperCase,
    LowerCase,
    #[default]
    Pass,
}
/// An atom represents a small piece of the output. We turn Tree-sitter nodes
/// into atoms, and we add white-space atoms where appropriate. The final list
/// of atoms is rendered to the output.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Atom {
    /// We don't allow consecutive `Hardline`, but a `Blankline` will render two
    /// newlines to produce a blank line.
    Blankline,
    /// A "no-op" atom that will not produce any output.
    #[default]
    Empty,
    /// Represents a newline.
    Hardline,
    /// Signals the end of an indentation block.
    IndentEnd,
    /// Signals the start of an indentation block. Any lines between the
    /// beginning and the end will be indented. In single-line constructs where
    /// the beginning and the end occurs on the same line, there will be no
    /// indentation.
    IndentStart,
    /// Represents the contents of a named Tree-sitter node. We track the node id here
    /// as well.
    Leaf {
        content: String,
        id: usize,
        original_position: Position,
        // marks the leaf to be printed on a single line, with no indentation
        single_line_no_indent: bool,
        // if the leaf is multi-line, each line will be indented, not just the first
        multi_line_indent_all: bool,
        // don't trim trailing newline characters if set to true
        keep_whitespace: bool,
        capitalisation: Capitalisation,
    },
    /// Represents a literal string, such as a semicolon.
    Literal(String),
    /// Represents a softline. It will be turned into a hardline for multi-line
    /// constructs, and either a space or nothing for single-line constructs.
    Softline {
        spaced: bool,
    },
    /// Represents a space. Consecutive spaces are reduced to one before rendering.
    Space,
    /// Represents the destruction of errant spaces. Adjacent consecutive spaces are
    /// reduced to zero before rendering.
    Antispace,
    /// Represents a segment to be deleted.
    // It is a segment, because if one wants to delete a node,
    // it might happen that it contains several leaves.
    DeleteBegin,
    DeleteEnd,

    CaseBegin(Capitalisation),
    CaseEnd,
    /// Indicates the beginning of a scope, use in combination with the
    /// ScopedSoftlines and ScopedConditionals below.
    ScopeBegin(ScopeInformation),
    /// Indicates the end of a scope, use in combination with the
    /// ScopedSoftlines and ScopedConditionals below.
    ScopeEnd(ScopeInformation),
    // Indicates the beginning of a *measuring* scope, that must be related to a "normal" one.
    // Used in combination with ScopedSoftlines and ScopedConditionals below.
    MeasuringScopeBegin(ScopeInformation),
    // Indicates the end of a *measuring* scope, that must be related to a "normal" one.
    // Used in combination with ScopedSoftlines and ScopedConditionals below.
    MeasuringScopeEnd(ScopeInformation),
    /// Scoped commands
    // ScopedSoftline works together with the @{prepend,append}_begin[_measuring]_scope and
    // @{prepend,append}_end[_measuring]_scope query tags. To decide if a scoped softline
    // must be expanded into a hardline, we look at the innermost scope having
    // the corresponding `scope_id`, that encompasses it. We expand the softline
    // if that scope is multi-line.
    // If that scope contains a *measuring* scope with the same `scope_id`, we expand
    // the node iff that *measuring* scope is multi-line.
    // The `id` value is here for technical reasons,
    // it allows tracking of the atom during post-processing.
    ScopedSoftline {
        id: usize,
        scope_id: String,
        spaced: bool,
    },
    /// Represents an atom that must only be output if the associated scope
    /// (or its associated measuring scope, see above) meets the condition
    /// (single-line or multi-line).
    ScopedConditional {
        id: usize,
        scope_id: String,
        condition: ScopeCondition,
        atom: Box<Atom>,
    },
}

impl Atom {
    /// This function is only expected to take spaces and newlines as argument.
    /// It defines the order Blankline > Hardline > Space > Empty.
    pub(crate) fn dominates(&self, other: &Atom) -> bool {
        match self {
            Atom::Empty => false,
            Atom::Space => matches!(other, Atom::Empty),
            Atom::Hardline => matches!(other, Atom::Space | Atom::Empty),
            Atom::Blankline => matches!(other, Atom::Hardline | Atom::Space | Atom::Empty),
            _ => panic!("Unexpected character in is_dominant"),
        }
    }
}

/// Used in `Atom::ScopedConditional` to apply the containing Atoms only if
/// the matched node spans a single line or multiple lines
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeCondition {
    /// The Atom is only applied if the matching node spans exactly one line
    SingleLineOnly,
    /// The Atom is only applied if the matching node spans two or more lines
    MultiLineOnly,
}

/// A convenience wrapper around `std::result::Result<T, FormatterError>`.
pub type FormatterResult<T, E = FormatterError> = Result<T, rootcause::Report<E>>;

/// Resolves an injected language name to a Topiary language definition.
///
/// The input is the language name declared by an injection query's
/// `#injection_language!` predicate.
///
/// Return `Ok(Some(language))` when the language is available. Return
/// `Ok(None)` when the resolver ran successfully but does not know that
/// language. Return `Err` when resolution failed, for example because a query
/// file, configuration entry, or grammar could not be loaded.
///
/// If a formatting operation encounters a matched injection, both `Ok(None)`
/// and `Err` are hard failures. Callers may pass no resolver only when
/// formatting languages without injection queries, or when using non-formatting
/// operations such as visualisation.
pub type LanguageResolver<'a> = dyn Fn(&str) -> FormatterResult<Option<Arc<Language>>> + 'a;

/// Operations that can be performed by the formatter.
#[derive(Clone, Copy, Debug)]
pub enum Operation {
    /// Formatting is the default operation of the formatter, it applies the
    /// formatting rules defined in the query file and outputs the result
    Format {
        /// If true, skips the idempotence check (where we format twice,
        /// succeeding only if the intermediate and final result are identical)
        skip_idempotence: bool,
        /// If true, Topiary will consider an ERROR as it does a leaf node,
        /// and continues formatting instead of exiting with an error
        tolerate_parsing_errors: bool,
    },
    /// Visualises the parsed file's tree-sitter tree
    Visualise {
        /// Choose the type of visualation Topiary should output
        output_format: Visualisation,
    },
}

/// The function that takes an input and formats, or visualises an output.
///
/// # Errors
///
/// If formatting fails for any reason, a `FormatterError` will be returned.
///
/// # Language injections
///
/// When formatting a language with an injection query, `resolve` must provide
/// definitions for every injected language that can be matched in the input.
/// Matched injections are all-or-nothing: if an injected language cannot be
/// resolved or its captured span cannot be formatted, this function returns an
/// error. Pass `None` only for languages without injection queries, or for
/// operations that do not format.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// use topiary_core::{formatter, Language, FormatterError, TopiaryQuery, Operation};
///
/// let input = "[1,2]".to_string();
/// let mut input = input.as_bytes();
/// let mut output = Vec::new();
///
/// // The grammar is loaded dynamically via `topiary-config` rather than
/// // depending on the `tree-sitter-json` crate directly.
/// let config = topiary_config::Configuration::default();
/// let grammar = config.get_language("json").unwrap().grammar().unwrap();
///
/// let language: Language = Language {
///     name: "json".to_owned(),
///     formatting_query: TopiaryQuery::new(&grammar, topiary_queries::json()).unwrap(),
///     grammar,
///     indent: None,
///     injection_query: None,
/// };
///
/// match formatter(&mut input, &mut output, &language, Operation::Format{ skip_idempotence: false, tolerate_parsing_errors: false }, None) {
///   Ok(()) => {
///     let formatted = String::from_utf8(output).expect("valid utf-8");
///   }
///   Err(r) => {
///     if let FormatterError::Query(message) = r.current_context() {
///         panic!("Error in query file: {message}");
///     }
///     panic!("An error occurred");
///   }
/// }
/// # }) // end tokio_test
/// ```
pub fn formatter(
    input: &mut impl io::Read,
    output: &mut impl io::Write,
    language: &Language,
    operation: Operation,
    resolve: Option<&LanguageResolver<'_>>,
) -> FormatterResult<()> {
    let content = read_input(input)
        .context_to()
        .attach("Failed to read input contents")?;

    formatter_str(&content, output, language, operation, resolve)
}

/// The function that takes a string slice and formats, or visualises an output.
///
/// # Errors
///
/// If formatting fails for any reason, a `FormatterError` will be returned.
///
/// # Language injections
///
/// See [`formatter`] for the `resolve` argument's semantics.
pub fn formatter_str(
    input: &str,
    output: &mut impl io::Write,
    language: &Language,
    operation: Operation,
    resolve: Option<&LanguageResolver<'_>>,
) -> FormatterResult<()> {
    let tolerate_parsing_errors = match operation {
        Operation::Format {
            tolerate_parsing_errors,
            ..
        } => tolerate_parsing_errors,
        _ => false,
    };

    let tree = tree_sitter::parse(input, &language.grammar, tolerate_parsing_errors)?;

    formatter_tree(tree, input, output, language, operation, resolve)?;

    Ok(())
}

/// The function that takes a tree and formats, or visualises an output.
///
/// # Errors
///
/// If formatting fails for any reason, a `FormatterError` will be returned.
///
/// # Language injections
///
/// See [`formatter`] for the `resolve` argument's semantics.
pub fn formatter_tree(
    tree: topiary_tree_sitter_facade::Tree,
    input_content: &str,
    output: &mut impl io::Write,
    language: &Language,
    operation: Operation,
    resolve: Option<&LanguageResolver<'_>>,
) -> FormatterResult<()> {
    match operation {
        Operation::Format {
            skip_idempotence,
            tolerate_parsing_errors,
        } => {
            log::debug!("Discovering potentially injected languages");
            let spans = match &language.injection_query {
                Some(injection_query) => collect_injections(&tree, input_content, injection_query),
                None => Vec::new(),
            };

            // Create a list of nodes that are injection formatted.
            // These must will be treated as leaves (although, in all likelihood, they already are).
            let injection_leaf_nodes = spans.iter().map(|span| span.node_id);

            // All the work related to tree-sitter and the query is done here
            log::debug!("Apply Tree-sitter query");

            let mut atoms = tree_sitter::apply_query_tree_with_forced_leaves(
                tree,
                input_content,
                &language.formatting_query,
                injection_leaf_nodes,
            )?;

            rewrite_injected_leaves(&mut atoms, spans, resolve, tolerate_parsing_errors)?;

            // Various post-processing of whitespace
            atoms.post_process();

            // Pretty-print atoms
            log::debug!("Pretty-print output");
            let rendered = pretty::render(
                &atoms[..],
                // Default to "  " if the language has no indentation specified
                language.indent.as_ref().map_or("  ", |v| v.as_str()),
            )?;

            // Add a final line break if missing
            let rendered = format!("{}\n", rendered.trim());

            if !skip_idempotence {
                idempotence_check(&rendered, language, tolerate_parsing_errors, resolve)?;
            }

            write!(output, "{rendered}").context_to()?;
        }

        Operation::Visualise { output_format } => {
            let root: SyntaxNode = tree.root_node().into();

            match output_format {
                Visualisation::GraphViz => graphviz::write(output, &root).context_to()?,
                Visualisation::Json => serde_json::to_writer(output, &root).context_to()?,
            };
        }
    };
    Ok(())
}

fn rewrite_injected_leaves(
    atoms: &mut atom_collection::AtomCollection,
    spans: Vec<InjectionSpan>,
    resolve: Option<&LanguageResolver<'_>>,
    tolerate_parsing_errors: bool,
) -> FormatterResult<()> {
    for span in spans {
        // If the injected language is unsupported, skip formatting this injection
        // by continuing the loop. This leaves the original, unformatted text intact.
        let Some(inner_language) = resolve_injected_language(resolve, &span.language)? else {
            log::warn!(
                "Skipping injection for unsupported language: {}",
                span.language
            );
            continue;
        };

        let mut formatted_inner = Vec::new();
        formatter_str(
            span.content,
            &mut formatted_inner,
            &inner_language,
            Operation::Format {
                skip_idempotence: true,
                tolerate_parsing_errors,
            },
            resolve,
        )?;

        let formatted_inner = String::from_utf8(formatted_inner)
            .context_to()?
            .trim_end_matches('\n')
            .to_owned();

        if !atoms.rewrite_injected_leaf_content(span.node_id, formatted_inner) {
            return Err(report!(FormatterError::Internal(format!(
                "Could not find leaf for injected {} span",
                span.language
            ))));
        }
    }

    Ok(())
}

/// Resolves a language string from an injection (e.g. "rust" in ```rust) into a `Language`
/// instance.
///
/// Returns `Ok(None)` if the language is not configured or cannot be resolved, allowing the caller
/// to gracefully skip formatting rather than failing the entire formatting operation.
fn resolve_injected_language(
    resolve: Option<&LanguageResolver<'_>>,
    language: &str,
) -> FormatterResult<Option<Arc<Language>>> {
    let Some(resolve) = resolve else {
        return Ok(None);
    };

    match resolve(language) {
        Ok(Some(language_cfg)) => {
            log::info!("resolved injected language: {language}");
            Ok(Some(language_cfg))
        }
        Ok(None) => Ok(None),
        Err(err)
            if matches!(
                err.current_context(),
                FormatterError::InjectionLanguageResolution { .. }
            ) =>
        {
            Err(err)
        }
        Err(err) => Err(err.context(FormatterError::InjectionLanguageResolution {
            language: language.to_owned(),
        })),
    }
}

/// Simple helper function to read the full content of an io Read stream
fn read_input(input: &mut dyn io::Read) -> Result<String, io::Error> {
    let mut content = String::new();
    input.read_to_string(&mut content)?;
    Ok(content)
}

/// Perform the idempotence check. Given the already formatted content of the
/// file, formats the content again and checks if the two are identical.
/// Result in: `Ok(())`` if the idempotence check succeeded (the content is
/// identical to the formatted content)
///
/// # Errors
///
/// `Err(FormatterError::Idempotence)` if the idempotence check failed
/// `Err(FormatterError::Formatting(...))` if the formatting failed
fn idempotence_check(
    content: &str,
    language: &Language,
    tolerate_parsing_errors: bool,
    resolve: Option<&LanguageResolver<'_>>,
) -> FormatterResult<()> {
    log::info!("Checking for idempotence ...");

    let mut input = content.as_bytes();
    let mut output = io::BufWriter::new(Vec::new());

    match formatter(
        &mut input,
        &mut output,
        language,
        Operation::Format {
            skip_idempotence: true,
            tolerate_parsing_errors,
        },
        resolve,
    ) {
        Ok(()) => {
            let reformatted = output
                .into_inner()
                .context_to()
                .map(String::from_utf8)?
                .context_to()?;

            if content == reformatted {
                Ok(())
            } else {
                log::error!("Failed idempotence check");
                log::error!("{}", StrComparison::new(content, &reformatted));
                Err(report!(FormatterError::Idempotence))
            }
        }
        Err(report) if matches!(report.current_context(), FormatterError::Parsing) => {
            Err(report.context(FormatterError::IdempotenceParsing))
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use test_log::test;

    use crate::{
        FormatterError, InjectionQuery, Language, Operation, SpanAttachment, TopiaryQuery,
        collect_injections, formatter, formatter_str, parse, test_utils::pretty_assert_eq,
    };

    fn language(name: &str, formatting_query: &str, injection_query: Option<&str>) -> Language {
        let config = topiary_config::Configuration::default();
        let config_language = config.get_language(name).unwrap();
        let grammar = config_language.grammar().unwrap();

        Language {
            name: name.to_owned(),
            formatting_query: TopiaryQuery::new(&grammar, formatting_query).unwrap(),
            injection_query: injection_query
                .map(|query_content| InjectionQuery::new(&grammar, query_content).unwrap()),
            grammar,
            indent: config_language.indent(),
        }
    }

    fn ocamllex_language() -> Language {
        language(
            "ocamllex",
            topiary_queries::ocamllex(),
            Some(topiary_queries::ocamllex_injections()),
        )
    }

    fn ocaml_language() -> Language {
        language("ocaml", topiary_queries::ocaml(), None)
    }

    fn unstable_ocaml_language() -> Language {
        language(
            "ocaml",
            r#"
((value_name) @append_delimiter
 (#delimiter! "x"))
"#,
            None,
        )
    }

    /// Attempt to parse invalid json, expecting a failure
    #[test(tokio::test)]
    async fn parsing_error_fails_formatting() {
        let mut input = r#"{"foo":{"bar"}}"#.as_bytes();
        let mut output = Vec::new();
        let language = language("json", "(#language! json)", None);

        let mut result = formatter(
            &mut input,
            &mut output,
            &language,
            Operation::Format {
                skip_idempotence: true,
                tolerate_parsing_errors: false,
            },
            None,
        );

        if let Some(range) = result
            .get_span()
            .and_then(|s| s.range)
            .inspect(|r| println!("{r:?}"))
            && range.start_point().row() == 0
            && range.end_point().row() == 0
        {
            return;
        }
        panic!("Expected a parsing error on line 1, but got {result:?}");
    }

    #[test(tokio::test)]
    async fn tolerate_parsing_errors() {
        // Contains the invalid object {"bar"   "baz"}. It should be left untouched.
        let mut input = "{\"one\":{\"bar\"   \"baz\"},\"two\":\"bar\"}".as_bytes();
        let expected = "{ \"one\": {\"bar\"   \"baz\"}, \"two\": \"bar\" }\n";

        let mut output = Vec::new();
        let language = language("json", topiary_queries::json(), None);

        formatter(
            &mut input,
            &mut output,
            &language,
            Operation::Format {
                skip_idempotence: true,
                tolerate_parsing_errors: true,
            },
            None,
        )
        .unwrap();

        let formatted = String::from_utf8(output).unwrap();
        log::debug!("{formatted}");

        pretty_assert_eq(expected, &formatted);
    }

    #[test(tokio::test)]
    async fn collect_injections_returns_content_span() {
        let input = r#"rule token = parse
  | "x" { let values=[1;2;3] in List.map (fun x->x+1) values }
"#;
        let language = ocamllex_language();
        let tree = parse(input, &language.grammar, false).unwrap();
        let spans = collect_injections(&tree, input, language.injection_query.as_ref().unwrap());

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].language, "ocaml");
        assert_eq!(
            spans[0].content,
            r#"let values=[1;2;3] in List.map (fun x->x+1) values"#
        );
    }

    #[test(tokio::test)]
    async fn unresolved_injection_skips_formatting() {
        let input = r#"rule token = parse
  | "x" { let values=[1;2;3] in List.map (fun x->x+1) values }
"#;
        let language = ocamllex_language();
        let mut output = Vec::new();

        let result = formatter_str(
            input,
            &mut output,
            &language,
            Operation::Format {
                skip_idempotence: true,
                tolerate_parsing_errors: false,
            },
            None,
        );

        assert!(result.is_ok());
    }

    #[test(tokio::test)]
    async fn resolver_error_fails_formatting() {
        let input = r#"rule token = parse
  | "x" { let values=[1;2;3] in List.map (fun x->x+1) values }
"#;
        let language = ocamllex_language();
        let mut output = Vec::new();

        let result = formatter_str(
            input,
            &mut output,
            &language,
            Operation::Format {
                skip_idempotence: true,
                tolerate_parsing_errors: false,
            },
            Some(&|_| {
                Err(rootcause::report!(FormatterError::Query(
                    "resolver failed while loading language".into()
                )))
            }),
        );

        assert!(matches!(
            result,
            Err(ref report)
                if matches!(
                    report.current_context(),
                    FormatterError::InjectionLanguageResolution { language } if language == "ocaml"
                )
        ));
    }

    #[test(tokio::test)]
    async fn resolved_injection_rewrites_forced_leaf() {
        let input = r#"rule token = parse
  | "x" { let values=[1;2;3] in List.map (fun x->x+1) values }
"#;
        let language = ocamllex_language();
        let inner_language: Arc<Language> = Arc::new(ocaml_language());
        let mut output = Vec::new();

        formatter_str(
            input,
            &mut output,
            &language,
            Operation::Format {
                skip_idempotence: true,
                tolerate_parsing_errors: false,
            },
            Some(&|name| Ok((name == "ocaml").then_some(inner_language.clone()))),
        )
        .unwrap();

        let formatted = String::from_utf8(output).unwrap();
        pretty_assert_eq(
            r#"rule token = parse
  | "x" { let values = [1; 2; 3] in List.map (fun x -> x + 1) values }"#,
            formatted.trim_end(),
        );
    }

    #[test(tokio::test)]
    async fn invalid_injected_source_fails_formatting() {
        let input = r#"rule token = parse
  | "x" { let x = }
"#;
        let language = ocamllex_language();
        let inner_language: Arc<Language> = Arc::new(ocaml_language());
        let mut output = Vec::new();

        let result = formatter_str(
            input,
            &mut output,
            &language,
            Operation::Format {
                skip_idempotence: true,
                tolerate_parsing_errors: false,
            },
            Some(&|name| Ok((name == "ocaml").then_some(inner_language.clone()))),
        );

        assert!(
            matches!(result, Err(ref report) if report.current_context() == &FormatterError::Parsing)
        );
    }

    #[test(tokio::test)]
    async fn non_idempotent_injection_fails_outer_idempotence() {
        let input = r#"rule token = parse
  | "x" { value }
"#;
        let language = ocamllex_language();
        let inner_language: Arc<Language> = Arc::new(unstable_ocaml_language());
        let mut output = Vec::new();

        let result = formatter_str(
            input,
            &mut output,
            &language,
            Operation::Format {
                skip_idempotence: false,
                tolerate_parsing_errors: false,
            },
            Some(&|name| Ok((name == "ocaml").then_some(inner_language.clone()))),
        );

        assert!(
            matches!(result, Err(ref report) if report.current_context() == &FormatterError::Idempotence)
        );
    }
}
