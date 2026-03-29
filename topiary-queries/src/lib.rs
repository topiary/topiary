/// Returns the Topiary-compatible query file for Bash.
#[cfg(feature = "bash")]
pub fn bash() -> &'static str {
    include_str!("../queries/bash.scm")
}

/// Returns the Topiary-compatible query file for CSS.
#[cfg(feature = "css")]
pub fn css() -> &'static str {
    include_str!("../queries/css.scm")
}

/// Returns the Topiary-compatible query file for Json.
#[cfg(feature = "json")]
pub fn json() -> &'static str {
    include_str!("../queries/json.scm")
}

/// Returns the Topiary-compatible query file for JSON5.
#[cfg(feature = "json5")]
pub fn json5() -> &'static str {
    include_str!("../queries/json5.scm")
}

/// Returns the Topiary-compatible query file for Nickel.
#[cfg(feature = "nickel")]
pub fn nickel() -> &'static str {
    include_str!("../queries/nickel.scm")
}

/// Returns the Topiary-compatible query file for Ocaml.
#[cfg(feature = "ocaml")]
pub fn ocaml() -> &'static str {
    include_str!("../queries/ocaml.scm")
}

/// Returns the Topiary-compatible query file for Ocaml Interface.
#[cfg(feature = "ocaml_interface")]
pub fn ocaml_interface() -> &'static str {
    include_str!("../queries/ocaml_interface.scm")
}

/// Returns the Topiary-compatible query file for Ocamllex.
#[cfg(feature = "ocamllex")]
pub fn ocamllex() -> &'static str {
    include_str!("../queries/ocamllex.scm")
}

/// Returns the Topiary-compatible query file for OpenSCAD.
#[cfg(feature = "openscad")]
pub fn openscad() -> &'static str {
    include_str!("../queries/openscad.scm")
}

/// Returns the Topiary-compatible query file for Rust.
#[cfg(feature = "rust")]
pub fn rust() -> &'static str {
    include_str!("../queries/rust.scm")
}

/// Returns the Topiary-compatible query file for SDML.
#[cfg(feature = "sdml")]
pub fn sdml() -> &'static str {
    include_str!("../queries/sdml.scm")
}

/// Returns the Topiary-compatible query file for Toml.
#[cfg(feature = "toml")]
pub fn toml() -> &'static str {
    include_str!("../queries/toml.scm")
}

/// Returns the Topiary-compatible query file for the
/// Tree-sitter query language.
#[cfg(feature = "tree_sitter_query")]
pub fn tree_sitter_query() -> &'static str {
    include_str!("../queries/tree_sitter_query.scm")
}

/// Returns the Topiary-compatible query file for WIT.
#[cfg(feature = "wit")]
pub fn wit() -> &'static str {
    include_str!("../queries/wit.scm")
}
