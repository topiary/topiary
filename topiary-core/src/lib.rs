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
//! [GitHub](https://github.com/tweag/topiary).

use std::io;

use pretty_assertions::StrComparison;
use rootcause::{prelude::ResultExt, report};
use tree_sitter::Position;

pub use crate::{
    error::{FormatterError, IoError},
    language::Language,
    tree_sitter::{
        CoverageData, SyntaxNode, TopiaryQuery, Visualisation, apply_query, check_query_coverage,
        parse,
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
pub type FormatterResult<T> = std::result::Result<T, rootcause::Report<FormatterError>>;

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
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// use std::fs::File;
/// use std::io::{BufReader, Read};
/// use topiary_core::{formatter, Language, FormatterError, TopiaryQuery, Operation};
///
/// let input = "[1,2]".to_string();
/// let mut input = input.as_bytes();
/// let mut output = Vec::new();
/// let json = topiary_tree_sitter_facade::Language::from(tree_sitter_json::LANGUAGE);
///
/// let mut query_file = BufReader::new(File::open("../topiary-queries/queries/json.scm").expect("query file"));
/// let mut query_content = String::new();
/// query_file.read_to_string(&mut query_content).expect("read query file");
///
/// let language: Language = Language {
///     name: "json".to_owned(),
///     query: TopiaryQuery::new(&json.clone().into(), &query_content).unwrap(),
///     grammar: json.into(),
///     indent: None,
/// };
///
/// match formatter(&mut input, &mut output, &language, Operation::Format{ skip_idempotence: false, tolerate_parsing_errors: false }) {
///   Ok(()) => {
///     let formatted = String::from_utf8(output).expect("valid utf-8");
///   }
///   Err(FormatterError::Query(message, _)) => {
///     panic!("Error in query file: {message}");
///   }
///   Err(_) => {
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
) -> FormatterResult<()> {
    let content = read_input(input)
        .context_to()
        .attach("Failed to read input contents")?;

    formatter_str(&content, output, language, operation)
}

/// The function that takes a string slice and formats, or visualises an output.
///
/// # Errors
///
/// If formatting fails for any reason, a `FormatterError` will be returned.
pub fn formatter_str(
    input: &str,
    output: &mut impl io::Write,
    language: &Language,
    operation: Operation,
) -> FormatterResult<()> {
    let tolerate_parsing_errors = match operation {
        Operation::Format {
            tolerate_parsing_errors,
            ..
        } => tolerate_parsing_errors,
        _ => false,
    };

    let tree = tree_sitter::parse(input, &language.grammar, tolerate_parsing_errors)?;

    formatter_tree(tree, input, output, language, operation)?;

    Ok(())
}

/// The function that takes a tree and formats, or visualises an output.
///
/// # Errors
///
/// If formatting fails for any reason, a `FormatterError` will be returned.
pub fn formatter_tree(
    tree: topiary_tree_sitter_facade::Tree,
    input_content: &str,
    output: &mut impl io::Write,
    language: &Language,
    operation: Operation,
) -> FormatterResult<()> {
    match operation {
        Operation::Format {
            skip_idempotence,
            tolerate_parsing_errors,
        } => {
            // All the work related to tree-sitter and the query is done here
            log::debug!("Apply Tree-sitter query");

            let mut atoms = tree_sitter::apply_query_tree(tree, input_content, &language.query)?;

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
                idempotence_check(&rendered, language, tolerate_parsing_errors)?;
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
        Err(report) if matches!(report.current_context(), FormatterError::Parsing { .. }) => {
            Err(report.context(FormatterError::IdempotenceParsing))
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use test_log::test;

    use crate::{
        Language, Operation, TopiaryQuery, error::FormatterError, formatter,
        test_utils::pretty_assert_eq,
    };

    /// Attempt to parse invalid json, expecting a failure
    #[test(tokio::test)]
    async fn parsing_error_fails_formatting() {
        let mut input = r#"{"foo":{"bar"}}"#.as_bytes();
        let mut output = Vec::new();
        let query_content = "(#language! json)";
        let grammar = topiary_tree_sitter_facade::Language::from(tree_sitter_json::LANGUAGE);
        let language = Language {
            name: "json".to_owned(),
            query: TopiaryQuery::new(&grammar, query_content).unwrap(),
            grammar,
            indent: None,
        };

        match formatter(
            &mut input,
            &mut output,
            &language,
            Operation::Format {
                skip_idempotence: true,
                tolerate_parsing_errors: false,
            },
        ) {
            // start end == 1
            Err(FormatterError::Parsing(node))
                if node.start_point().row() == 0 && node.end_point().row() == 0 => {}
            result => {
                panic!("Expected a parsing error on line 1, but got {result:?}");
            }
        }
    }

    #[test(tokio::test)]
    async fn tolerate_parsing_errors() {
        // Contains the invalid object {"bar"   "baz"}. It should be left untouched.
        let mut input = "{\"one\":{\"bar\"   \"baz\"},\"two\":\"bar\"}".as_bytes();
        let expected = "{ \"one\": {\"bar\"   \"baz\"}, \"two\": \"bar\" }\n";

        let mut output = Vec::new();
        let query_content = fs::read_to_string("../topiary-queries/queries/json.scm").unwrap();
        let grammar = tree_sitter_json::LANGUAGE.into();
        let language = Language {
            name: "json".to_owned(),
            query: TopiaryQuery::new(&grammar, &query_content).unwrap(),
            grammar,
            indent: None,
        };

        formatter(
            &mut input,
            &mut output,
            &language,
            Operation::Format {
                skip_idempotence: true,
                tolerate_parsing_errors: true,
            },
        )
        .unwrap();

        let formatted = String::from_utf8(output).unwrap();
        log::debug!("{formatted}");

        pretty_assert_eq(expected, &formatted);
    }
}
