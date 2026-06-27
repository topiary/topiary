use std::fmt;

use crate::{InjectionQuery, TopiaryQuery};

/// A Language contains all the information Topiary requires to format that
/// specific languages.
#[derive(Debug)]
pub struct Language {
    /// The name of the language, used as a key when looking up information in
    /// the Configuration, and to convert from a language to the respective tree-sitter
    /// grammar.
    pub name: String,
    /// The Query Topiary will use to get the formatting captures, must be
    /// present. The topiary engine does not include any formatting queries.
    pub formatting_query: TopiaryQuery,
    /// Optional injection query identifying regions of source that should be
    /// formatted as a different language.
    pub injection_query: Option<InjectionQuery>,
    /// The tree-sitter Language. Topiary will use this Language for parsing.
    pub grammar: topiary_tree_sitter_facade::Language,
    /// The indentation string used for that particular language. Defaults to "  "
    /// if not provided. Any string can be provided, but in most instances will be
    /// some whitespace: "  ", "    ", or "\t".
    pub indent: Option<String>,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
