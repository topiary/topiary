// WASM build doesn't use topiary_tree_sitter_facade::QueryMatch or
// streaming_iterator::StreamingIterator
#![cfg_attr(target_arch = "wasm32", allow(unused_imports))]

use std::{collections::HashSet, fmt::Display};

use miette::{LabeledSpan, Severity, SourceSpan};
use rootcause::{prelude::ResultExt, report};
use serde::Serialize;

use topiary_tree_sitter_facade::{
    Node, Parser, Point, Query, QueryCapture, QueryCursor, QueryError, QueryMatch, QueryPredicate,
    Tree,
};

use streaming_iterator::StreamingIterator;

use crate::{
    FormatterResult,
    atom_collection::{AtomCollection, QueryPredicates},
    error::{FormatterError, SpanAttachment},
};

/// Supported visualisation formats
#[derive(Clone, Copy, Debug)]
pub enum Visualisation {
    GraphViz,
    Json,
}

/// Refers to a position within the code. Used for error reporting, and for
/// comparing input with formatted output. The numbers are 1-based, because that
/// is how editors usually refer to a position. Derived from tree_sitter::Point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Position {
    pub row: u32,
    pub column: u32,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "({},{})", self.row, self.column)
    }
}

/// Topiary often needs both the tree-sitter `Query` and the original content
/// belonging to the file from which the query was parsed. This struct is a simple
/// convenience wrapper that combines the `Query` with its original string.
#[derive(Debug)]
pub struct TopiaryQuery {
    pub query: Query,
    pub query_content: String,
}

impl TopiaryQuery {
    /// Creates a new `TopiaryQuery` from a tree-sitter language/grammar and the
    /// contents of the query file.
    ///
    /// # Errors
    ///
    /// This function will return an error if tree-sitter failed to parse the
    /// query file.
    pub fn new(
        grammar: &topiary_tree_sitter_facade::Language,
        query_content: &str,
    ) -> FormatterResult<TopiaryQuery, QueryError> {
        let query = Query::new(grammar, query_content)
            .into_report()
            .map_err(|e| {
                let range = e.current_context().range;
                e.attach_range(range)
            })
            .attach_source(Some(query_content))?;

        Ok(TopiaryQuery {
            query,
            query_content: query_content.to_owned(),
        })
    }

    /// Calculates the provided position of the Pattern in the query source file
    /// from the byte offset of the pattern in the query.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn pattern_position(&self, pattern_index: usize) -> Position {
        let byte_offset = self.query.start_byte_for_pattern(pattern_index);
        let (row, column) =
            self.query_content[..byte_offset]
                .chars()
                .fold((0, 0), |(row, column), c| {
                    if c == '\n' {
                        (row + 1, 0)
                    } else {
                        (row, column + 1)
                    }
                });
        Position {
            row: row + 1,
            column: column + 1,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn pattern_position(&self, _pattern_index: usize) -> Position {
        unimplemented!()
    }
}

/// A pre-compiled query that identifies regions of a parsed source which
/// should be formatted as a different ("injected") language.
///
/// An injection query captures the embedded source text with
/// `@injection.content`, and declares the inner language via a
/// `(#injection_language! "name")` predicate on the same pattern. For example,
/// to mark every `(ocaml)` node within an `ocamllex` source as OCaml:
///
/// ```scheme
/// ((ocaml) @injection.content
///  (#injection_language! "ocaml"))
/// ```
#[derive(Debug)]
pub struct InjectionQuery {
    pub query: Query,
    pub query_content: String,
}

impl InjectionQuery {
    /// Compile `query_content` against `grammar` as an injection query.
    ///
    /// # Errors
    ///
    /// Returns [`FormatterError::Query`] if tree-sitter fails to parse the
    /// query.
    pub fn new(
        grammar: &topiary_tree_sitter_facade::Language,
        query_content: &str,
    ) -> FormatterResult<InjectionQuery> {
        let query = Query::new(grammar, query_content)
            .into_report()
            .map_err(|e| {
                let range = e.current_context().range;
                e.attach_range(range)
            })
            .attach_source(Some(query_content))
            .context(FormatterError::Query(
                "Error parsing injection query file".into(),
            ))?;

        Ok(InjectionQuery {
            query,
            query_content: query_content.to_owned(),
        })
    }
}

/// A region of host source text that should be formatted as a different
/// language, as determined by an [`InjectionQuery`].
#[derive(Clone, Debug)]
pub struct InjectionSpan {
    pub byte_range: std::ops::Range<usize>,
    /// The injected language name, taken from the `#injection_language!`
    /// predicate of the matching pattern.
    pub language: String,
    /// Tree-sitter id of the captured node. Valid only against the same
    /// [`Tree`] from which these spans were collected: tree-sitter does not
    /// guarantee node-id stability across edit-and-reparse, but within a
    /// single parse IDs are stable. Used to locate and rewrite the
    /// corresponding `Atom::Leaf` after the host has been atomised.
    pub node_id: usize,
}

/// Run an [`InjectionQuery`] against a parsed `tree`, returning every
/// `@injection.content` capture paired with the language declared by its
/// pattern's `#injection_language!` predicate.
///
/// Patterns missing an `#injection_language!` predicate are skipped (with a
/// warning logged).
///
/// Missing predicates or unmatched captures are logged, not raised.
pub fn collect_injections(
    tree: &Tree,
    input_content: &str,
    query: &InjectionQuery,
) -> Vec<InjectionSpan> {
    let root = tree.root_node();
    let source = input_content.as_bytes();
    let capture_names = query.query.capture_names();

    let mut cursor = QueryCursor::new();
    let mut spans = Vec::new();

    let mut matches = query.query.matches(&root, source, &mut cursor);
    #[allow(clippy::while_let_on_iterator)] // Not a normal iterator
    while let Some(query_match) = matches.next() {
        // Resolve the language of the injection either via a hardcoded `#injection_language!` predicate
        // or by dynamically reading the text of the `@injection.language` capture (e.g. for Markdown code blocks).
        let language_name = query
            .query
            .general_predicates(query_match.pattern_index())
            .into_iter()
            .find_map(|p| {
                (&*p.operator() == "injection_language!")
                    .then(|| p.args().into_iter().next())
                    .flatten()
            })
            .map(|s| s.to_string())
            .or_else(|| {
                query_match
                    .captures()
                    .find(|c| c.name(capture_names.as_slice()) == "injection.language")
                    .and_then(|c| c.node().utf8_text(source).ok())
                    .map(|s| s.to_string())
            });

        let Some(language_name) = language_name else {
            log::warn!(
                "Injection query pattern {} has neither an #injection_language! predicate nor an @injection.language capture; skipping",
                query_match.pattern_index()
            );
            continue;
        };

        for capture in query_match
            .captures()
            .filter(|c| c.name(capture_names.as_slice()) == "injection.content")
        {
            let node = capture.node();
            spans.push(InjectionSpan {
                byte_range: node.start_byte() as usize..node.end_byte() as usize,
                language: language_name.clone(),
                node_id: node.id(),
            });
        }
    }

    spans
}

impl From<Point> for Position {
    fn from(point: Point) -> Self {
        Self {
            row: point.row() + 1,
            column: point.column() + 1,
        }
    }
}

// Simplified syntactic node struct, for the sake of serialisation.
#[derive(Serialize)]
pub struct SyntaxNode {
    #[serde(skip_serializing)]
    pub id: usize,

    pub kind: String,
    pub is_named: bool,
    is_extra: bool,
    is_error: bool,
    is_missing: bool,
    start: Position,
    end: Position,

    pub children: Vec<SyntaxNode>,
}

impl From<Node<'_>> for SyntaxNode {
    fn from(node: Node) -> Self {
        let mut walker = node.walk();
        let children = node.children(&mut walker).map(Self::from).collect();

        Self {
            id: node.id(),

            kind: node.kind().into(),
            is_named: node.is_named(),
            is_extra: node.is_extra(),
            is_error: node.is_error(),
            is_missing: node.is_missing(),
            start: node.start_position().into(),
            end: node.end_position().into(),

            children,
        }
    }
}

/// Extension trait for [`Node`] to allow for 1-based display in logs.
///
/// (Can't be done as a [`Display`] impl on [`Node`] directly, since that would
/// run into orphan issues. An alternative that would work is a [`Display`] impl
/// on a wrapper struct.)
pub trait NodeExt {
    /// Produce a textual representation with 1-based row/column indexes.
    fn display_one_based(&self) -> String;
}

impl NodeExt for Node<'_> {
    fn display_one_based(&self) -> String {
        format!(
            "{{Node {:?} {} - {}}}",
            self.kind(),
            Position::from(self.start_position()),
            Position::from(self.end_position()),
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl NodeExt for tree_sitter::Node<'_> {
    fn display_one_based(&self) -> String {
        format!(
            "{{Node {:?} {} - {}}}",
            self.kind(),
            Position::from(<tree_sitter::Point as Into<Point>>::into(
                self.start_position()
            )),
            Position::from(<tree_sitter::Point as Into<Point>>::into(
                self.end_position()
            )),
        )
    }
}

#[derive(Debug)]
// A struct to statically store the public fields of query match results,
// to avoid running queries twice.
struct LocalQueryMatch<'a> {
    pattern_index: usize,
    captures: Vec<QueryCapture<'a>>,
}

impl Display for LocalQueryMatch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "LocalQueryMatch {{ pattern_index: {}, captures: [ ",
            self.pattern_index
        )?;
        for (index, capture) in self.captures.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            // .node() doesn't provide access to the inner [`tree_sitter`]
            // object. As a result, we can't get the index out directly, so we
            // skip it for now.
            write!(f, "{}", capture.node().display_one_based())?;
        }
        write!(f, " ] }}")?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
// A struct to store the result of a query coverage check
pub struct CoverageData {
    pub cover_percentage: f32,
    pub missing_patterns: Vec<LabeledSpan>,
}

impl CoverageData {
    fn status_msg(&self) -> String {
        match self.cover_percentage {
            0.0 if self.missing_patterns.is_empty() => "No queries found".into(),
            1.0 => "All queries are matched".into(),
            _ => format!("Unmatched queries: {}", self.missing_patterns.len()),
        }
    }

    fn full_coverage(&self) -> bool {
        self.cover_percentage == 1.0
    }

    /// Returns an error if coverage is not 100%
    pub fn get_result(&self) -> FormatterResult<()> {
        if !self.full_coverage() {
            return Err(FormatterError::PatternDoesNotMatch.into());
        }
        Ok(())
    }
}

impl std::fmt::Display for CoverageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.status_msg())
    }
}
impl std::error::Error for CoverageData {}

impl miette::Diagnostic for CoverageData {
    fn severity(&self) -> Option<miette::Severity> {
        match self.cover_percentage {
            1.0 => Severity::Advice,
            0.0 => Severity::Warning,
            _ => Severity::Error,
        }
        .into()
    }
    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        Some(Box::new(self.missing_patterns.iter().cloned()))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        let msg = format!("Query coverage: {:.2}%", self.cover_percentage * 100.0);

        Some(Box::new(msg))
    }
}

/// Applies a query to an input content and returns a collection of atoms.
///
/// # Errors
///
/// This function can return an error if:
/// - The input content cannot be parsed by the grammar.
/// - The query content cannot be parsed by the grammar.
/// - The input exhaustivity check fails.
/// - A found predicate could not be parsed or is malformed.
/// - A unknown capture name was encountered in the query.
pub fn apply_query(
    input_content: &str,
    query: &TopiaryQuery,
    grammar: &topiary_tree_sitter_facade::Language,
    tolerate_parsing_errors: bool,
) -> FormatterResult<AtomCollection> {
    let tree = parse(input_content, grammar, tolerate_parsing_errors)?;
    apply_query_tree(tree, input_content, query)
}

/// Applies tree-sitter formatting queries to a parsed syntax tree, producing
/// an [`AtomCollection`] ready for rendering.
///
/// This is the second stage of the formatting pipeline, between
/// [`parse`] and [`render`](crate::pretty::render). It performs the
/// following steps:
///
/// 1. Match: Runs the query patterns against the tree's root node,
///    collecting all matches and their captures.
/// 2. Identify leaves: Finds nodes marked with `@leaf` in the query so
///    the flattening step will not recurse into them.
/// 3. Flatten: Converts the tree into a flat [`AtomCollection`] of
///    terminal/leaf nodes.
/// 4. Apply formatting: Processes each match and evaluates predicates
///    (e.g. `#single_line_only!`) and applies formatting directives
///    (`@append_space`, `@prepend_hardline`, etc.) to the corresponding atoms.
///
/// # Errors
///
/// This function can return an error if:
/// - The input exhaustivity check fails.
/// - A predicate could not be parsed or is malformed.
/// - An unknown capture name was encountered in the query.
pub fn apply_query_tree(
    tree: Tree,
    input_content: &str,
    query: &TopiaryQuery,
) -> FormatterResult<AtomCollection> {
    apply_query_tree_with_forced_leaves(tree, input_content, query, std::iter::empty())
}

pub(crate) fn apply_query_tree_with_forced_leaves(
    tree: Tree,
    input_content: &str,
    query: &TopiaryQuery,
    forced_leaf_nodes: impl Iterator<Item = usize>,
) -> FormatterResult<AtomCollection> {
    let root = tree.root_node();
    let source = input_content.as_bytes();

    // Match queries
    let mut cursor = QueryCursor::new();
    let mut matches: Vec<LocalQueryMatch> = Vec::new();
    let capture_names = query.query.capture_names();

    let mut query_matches = query.query.matches(&root, source, &mut cursor);
    #[allow(clippy::while_let_on_iterator)] // This is not a normal iterator
    while let Some(query_match) = query_matches.next() {
        let local_captures: Vec<QueryCapture> = query_match.captures().collect();

        matches.push(LocalQueryMatch {
            pattern_index: query_match.pattern_index(),
            captures: local_captures,
        });
    }

    // Find the ids of all tree-sitter nodes that were identified as a leaf
    // We want to avoid recursing into them in the collect_leaves function.
    let mut specified_leaf_nodes: HashSet<usize> =
        collect_leaf_ids(&matches, capture_names.clone());
    specified_leaf_nodes.extend(forced_leaf_nodes);

    // The Flattening: collects all terminal nodes of the tree-sitter tree in a Vec
    let mut atoms = AtomCollection::collect_leaves(&root, source, specified_leaf_nodes)?;

    log::debug!("List of atoms before formatting: {atoms:?}");

    // Memoization of the pattern positions
    let mut pattern_positions: Vec<Option<Position>> = Vec::new();

    // The web bindings for tree-sitter do not have support for pattern_count, so instead we will resize as needed
    // Only reallocate if we are actually going to use the vec
    #[cfg(not(target_arch = "wasm32"))]
    if log::log_enabled!(log::Level::Info) {
        pattern_positions.resize(query.query.pattern_count(), None);
    }

    // If there are more than one capture per match, it generally means that we
    // want to use the last capture. For example
    // (
    //   (enum_item) @append_hardline .
    //   (line_comment)? @append_hardline
    // )
    // means we want to append a hardline at
    // the end, but we don't know if we get a line_comment capture or not.
    for m in matches {
        let mut predicates = QueryPredicates::default();

        for p in query.query.general_predicates(m.pattern_index) {
            predicates = handle_predicate(&p, &predicates)?;
        }
        check_predicates(&predicates)?;

        // NOTE: Only performed if logging is enabled to avoid unnecessary computation of Position
        if log::log_enabled!(log::Level::Info) {
            #[cfg(target_arch = "wasm32")]
            // Resize the pattern_positions vector if we need to store more positions
            if m.pattern_index >= pattern_positions.len() {
                pattern_positions.resize(m.pattern_index + 1, None);
            }

            // Fetch from pattern_positions, otherwise insert
            let pos = pattern_positions[m.pattern_index].unwrap_or_else(|| {
                let pos = query.pattern_position(m.pattern_index);
                pattern_positions[m.pattern_index] = Some(pos);
                pos
            });

            let query_name_info = if let Some(name) = &predicates.query_name {
                format!(" of query \"{name}\"")
            } else {
                "".into()
            };

            log::debug!("Processing match{query_name_info}: {m} at location {pos}");
        }

        // If any capture is a do_nothing, then do nothing.
        if m.captures
            .iter()
            .any(|c| c.name(capture_names.as_slice()) == "do_nothing")
        {
            continue;
        }

        for c in m.captures {
            let name = c.name(capture_names.as_slice());
            atoms.resolve_capture(&name, &c.node(), &predicates)?;
        }
    }

    // Now apply all atoms in prepend and append to the leaf nodes.
    atoms.apply_prepends_and_appends();

    Ok(atoms)
}

/// Parses source code into a tree-sitter syntax tree.
///
/// This is the first stage of the formatting pipeline. It creates a
/// tree-sitter parser, applies the given grammar, and parses the input
/// into a concrete syntax tree. Unless `tolerate_parsing_errors` is set,
/// the resulting tree is checked for error nodes to ensure it is complete.
///
/// # Errors
///
/// Returns an error if the grammar cannot be applied, the input cannot
/// be parsed, or the resulting tree contains error nodes (when
/// `tolerate_parsing_errors` is `false`).
pub fn parse(
    content: &str,
    grammar: &topiary_tree_sitter_facade::Language,
    tolerate_parsing_errors: bool,
) -> FormatterResult<Tree> {
    let mut parser = Parser::new().context_to()?;
    parser
        .set_language(grammar)
        .context_to()
        .attach("Could not apply Tree-sitter grammar")?;

    let tree = parser.parse(content, None).context_to()?.ok_or_else(|| {
        report!(FormatterError::Internal(
            "Could not parse input".to_string()
        ))
    })?;

    // Fail parsing if we don't get a complete syntax tree.
    if !tolerate_parsing_errors {
        check_for_error_nodes(&tree.root_node()).attach_source(Some(content))?;
    }

    Ok(tree)
}

// returns first error node encountered
fn check_for_error_nodes(node: &Node) -> FormatterResult<()> {
    if node.is_error() {
        return Err(report!(FormatterError::Parsing)
            .attach_range(node.range())
            .attach_language(node.language_name()));
    }

    for child in node.children(&mut node.walk()) {
        check_for_error_nodes(&child)?;
    }

    Ok(())
}

/// Collects the IDs of all leaf nodes in a set of query matches.
///
/// This function takes a slice of `LocalQueryMatch` and a slice of capture names,
/// and returns a `HashSet` of node IDs that are matched by the "leaf" capture name.
fn collect_leaf_ids(matches: &[LocalQueryMatch], capture_names: Vec<&str>) -> HashSet<usize> {
    let mut ids = HashSet::new();

    for m in matches {
        for c in &m.captures {
            if c.name(capture_names.as_slice()) == "leaf" {
                ids.insert(c.node().id());
            }
        }
    }
    ids
}

/// Handles a query predicate and returns a new set of query predicates with the corresponding field updated.
///
/// # Arguments
///
/// * `predicate` - A reference to a `QueryPredicate` object that represents a predicate in a query pattern.
/// * `predicates` - A reference to a `QueryPredicates` object that holds the current state of the query predicates.
///
/// # Returns
///
/// A `FormatterResult` that contains either a new `QueryPredicates` object with the updated field, or a `FormatterError` if the predicate is invalid or missing an argument.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The predicate operator is not one of the supported ones.
/// * The predicate operator requires an argument but none is provided.
fn handle_predicate(
    predicate: &QueryPredicate,
    predicates: &QueryPredicates,
) -> FormatterResult<QueryPredicates> {
    let operator = &*predicate.operator();
    if "delimiter!" == operator {
        let arg = predicate
            .args()
            .into_iter()
            .next()
            .ok_or_else(|| FormatterError::Query(format!("{operator} needs an argument")))?;
        Ok(QueryPredicates {
            delimiter: Some(arg),
            ..predicates.clone()
        })
    } else if "scope_id!" == operator {
        let arg = predicate
            .args()
            .into_iter()
            .next()
            .ok_or_else(|| FormatterError::Query(format!("{operator} needs an argument")))?;
        Ok(QueryPredicates {
            scope_id: Some(arg),
            ..predicates.clone()
        })
    } else if "single_line_only!" == operator {
        Ok(QueryPredicates {
            single_line_only: true,
            ..predicates.clone()
        })
    } else if "multi_line_only!" == operator {
        Ok(QueryPredicates {
            multi_line_only: true,
            ..predicates.clone()
        })
    } else if "single_line_scope_only!" == operator {
        let arg = predicate
            .args()
            .into_iter()
            .next()
            .ok_or_else(|| FormatterError::Query(format!("{operator} needs an argument")))?;
        Ok(QueryPredicates {
            single_line_scope_only: Some(arg),
            ..predicates.clone()
        })
    } else if "multi_line_scope_only!" == operator {
        let arg = predicate
            .args()
            .into_iter()
            .next()
            .ok_or_else(|| FormatterError::Query(format!("{operator} needs an argument")))?;
        Ok(QueryPredicates {
            multi_line_scope_only: Some(arg),
            ..predicates.clone()
        })
    } else if "query_name!" == operator {
        let arg = predicate
            .args()
            .into_iter()
            .next()
            .ok_or_else(|| FormatterError::Query(format!("{operator} needs an argument")))?;
        Ok(QueryPredicates {
            query_name: Some(arg),
            ..predicates.clone()
        })
    } else {
        Err(FormatterError::Query(format!(
            "{operator} is an unknown predicate. Maybe you forgot a \"!\"?"
        )))
        .into_report()
    }
}

/// Checks the validity of the query predicates.
///
/// This function ensures that the query predicates do not contain more than one
/// of the following: #single_line_only, #multi_line_only, #single_line_scope_only,
/// or #multi_line_scope_only. These predicates are incompatible with each other
/// and would result in an invalid query.
///
/// # Arguments
///
/// * `predicates` - A reference to a QueryPredicates struct that holds the query predicates.
///
/// # Errors
///
/// If the query predicates contain more than one incompatible predicate, this function
/// returns a FormatterError::Query with a descriptive message.
fn check_predicates(predicates: &QueryPredicates) -> FormatterResult<()> {
    let mut incompatible_predicates = 0;
    if predicates.single_line_only {
        incompatible_predicates += 1;
    }
    if predicates.multi_line_only {
        incompatible_predicates += 1;
    }
    if predicates.single_line_scope_only.is_some() {
        incompatible_predicates += 1;
    }
    if predicates.multi_line_scope_only.is_some() {
        incompatible_predicates += 1;
    }
    if incompatible_predicates > 1 {
        Err(FormatterError::Query(
            "A query can contain at most one #single/multi_line[_scope]_only! predicate".into(),
        )
        .into())
    } else {
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Check if the input tests all patterns in the query, by successively disabling
/// all patterns. If disabling a pattern does not decrease the number of matches,
/// then that pattern originally matched nothing in the input.
pub fn check_query_coverage(
    input_content: &str,
    original_query: &TopiaryQuery,
    grammar: &topiary_tree_sitter_facade::Language,
) -> FormatterResult<CoverageData> {
    use miette::LabeledSpan;
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    let tree = parse(input_content, grammar, false)?;
    let root = tree.root_node();
    let source = input_content.as_bytes();
    let mut missing_patterns = Vec::new();

    // Match queries
    let mut cursor = QueryCursor::new();
    let ref_match_count = original_query
        .query
        .matches(&root, source, &mut cursor)
        .count();

    let pattern_count = original_query.query.pattern_count();
    let query_content = &original_query.query_content;
    let query = &original_query.query;

    // If there are no queries at all (e.g., when debugging) return early
    // rather than dividing by zero
    if pattern_count == 0 {
        let cover_percentage = 0.0;
        return Ok(CoverageData {
            cover_percentage,
            missing_patterns,
        });
    }

    // This particular test avoids a SIGSEGV error that occurs when trying
    // to count the matches of an empty query (see #481)
    if pattern_count == 1 {
        let mut cover_percentage = 1.0;
        if ref_match_count == 0 {
            missing_patterns.push(LabeledSpan::new_with_span(
                Some("empty query".into()),
                SourceSpan::from(0..query_content.len()),
            ));
            cover_percentage = 0.0
        }
        return Ok(CoverageData {
            cover_percentage,
            missing_patterns,
        });
    }

    let missing_patterns: Vec<LabeledSpan> = (0..pattern_count)
        .into_par_iter()
        .filter_map(|i| {
            // The TreeSitter API doesn't support splitting a query per pattern subqueries.
            // We do so manually here by using the `query_content` and `query` fields for the same
            // `TopiaryQuery` object.

            let start_idx = query.start_byte_for_pattern(i);
            let end_idx = query.end_byte_for_pattern(i);
            // SAFETY: the index range provided is returned directly from the inner `Query` object
            let pattern_content = unsafe { query_content.get_unchecked(start_idx..end_idx) };
            // All child patterns of a non-empty `Query` object created through `Query::new` are guaranteed
            // to create their own valid `Query` by referencing their pattern byte range.
            let pattern_query = Query::new(grammar, pattern_content)
                .expect("unable to create subquery of valid query, this is a bug");

            let mut cursor = QueryCursor::new();
            let pattern_has_matches = pattern_query
                .matches(&root, source, &mut cursor)
                .next()
                .is_some();
            if !pattern_has_matches {
                let trimmed_end_idx = pattern_content
                    .rmatch_indices('\n')
                    .map(|(i, _)| i)
                    .find_map(|i| {
                        let line = pattern_content[i..].trim_start();
                        let is_pattern_line = !line.is_empty() && !line.starts_with(';');
                        is_pattern_line.then_some(start_idx + i + 2)
                    })
                    .unwrap_or(pattern_content.len());
                return Some(LabeledSpan::new_with_span(
                    Some("unmatched".into()),
                    SourceSpan::from(start_idx..trimmed_end_idx),
                ));
            }
            None
        })
        .collect();

    let ok_patterns = pattern_count - missing_patterns.len();
    let cover_percentage = ok_patterns as f32 / pattern_count as f32;
    Ok(CoverageData {
        cover_percentage,
        missing_patterns,
    })
}

#[cfg(target_arch = "wasm32")]
pub fn check_query_coverage(
    _input_content: &str,
    _original_query: &TopiaryQuery,
    _grammar: &topiary_tree_sitter_facade::Language,
) -> FormatterResult<CoverageData> {
    unimplemented!();
}
