use std::{fmt, sync::Once};
#[cfg(any(
    feature = "json",
    feature = "toml",
    all(feature = "ocamllex", feature = "ocaml")
))]
use std::{fs, path::PathBuf};

use assert_cmd::cargo_bin_cmd;
use std::{fs::File, io::Write};
use tempfile::TempDir;

// Simple exemplar JSON and TOML state, to verify the formatter
// is doing something... and hopefully the right thing
#[cfg(feature = "json")]
const JSON_INPUT: &str = r#"{   "test"  :123}"#;
#[cfg(feature = "json")]
const JSON_EXPECTED: &str = r#"{ "test": 123 }
"#;

#[cfg(feature = "toml")]
const TOML_INPUT: &str = r#"   test=    123"#;
#[cfg(feature = "toml")]
const TOML_EXPECTED: &str = r#"test = 123
"#;

// We need to prefetch JSON and TOML grammars before running the tests, on pain of race condition:
// If multiple calls to Topiary are made in parallel and the grammar is missing, they will all try
// to fetch and build it, thus creating an empty .so file while g++ is running. If another instance
// of topiary starts at this moment, it will mistake the empty .so file for an already built grammar,
// and try to run with it, resulting in an error. See https://github.com/topiary/topiary/issues/767
static INIT: Once = Once::new();
pub fn initialize() {
    INIT.call_once(|| {
        #[cfg(feature = "json")]
        cargo_bin_cmd!("topiary")
            .arg("fmt")
            .arg("--language")
            .arg("json")
            .write_stdin("")
            .assert()
            .success();
        #[cfg(feature = "toml")]
        cargo_bin_cmd!("topiary")
            .arg("fmt")
            .arg("--language")
            .arg("toml")
            .write_stdin("")
            .assert()
            .success();
    });
}

// The TempDir member of the State is not actually used.
// However, removing it means that the directory is dropped at the end of the new() function, which causes it to be deleted.
// This causes the path to the state file to be invalid and breaks the tests.
// So, we keep the TempDir around so the tests don't break.
#[cfg(any(feature = "json", feature = "toml"))]
#[allow(dead_code)]
struct State(TempDir, PathBuf);

#[cfg(any(feature = "json", feature = "toml"))]
impl State {
    fn new(payload: &str, extension: &str) -> Self {
        let tmp_dir = TempDir::new().unwrap();
        let tmp_file = tmp_dir.path().join(format!("state.{extension}"));

        let mut state = File::create(&tmp_file).unwrap();
        write!(state, "{payload}").unwrap();

        Self(tmp_dir, tmp_file)
    }

    fn path(&self) -> &PathBuf {
        &self.1
    }

    fn read(&self) -> String {
        fs::read_to_string(self.path()).unwrap()
    }
}

#[test]
#[cfg(feature = "json")]
fn test_fmt_stdin() {
    initialize();
    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg("--language")
        .arg("json")
        .write_stdin(JSON_INPUT)
        .assert()
        .success()
        .stdout(JSON_EXPECTED);
}

#[test]
#[cfg(feature = "json")]
fn test_fmt_stdin_query() {
    initialize();
    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg("--language")
        .arg("json")
        .arg("--query")
        .arg(format!(
            "../topiary-queries/queries/json/{}.scm",
            topiary_queries::FORMATTING_QUERY
        ))
        .write_stdin(JSON_INPUT)
        .assert()
        .success()
        .stdout(JSON_EXPECTED);
}

// NOTE `test_fmt_stdin_query`, above, passes the built-in query as the override, so it
// cannot distinguish an honoured override from an ignored one. These two tests use an
// override that is observably *not* the built-in query, which is what a regression in the
// override plumbing actually looks like. See issue #1306.

#[test]
#[cfg(feature = "json")]
fn test_fmt_stdin_query_override_is_used() {
    use predicates::{prelude::PredicateBooleanExt, str::contains};

    // A deliberately idiosyncratic query: unlike the built-in JSON query, it puts a space
    // *before* the colon and adds no soft lines or indentation. Its output is therefore
    // unmistakably distinct from `JSON_EXPECTED`.
    let tmp_dir = TempDir::new().unwrap();
    let query = tmp_dir.path().join("formatting.scm");
    fs::write(&query, "(string) @leaf\n\":\" @prepend_space\n").unwrap();

    initialize();
    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg("--language")
        .arg("json")
        .arg("--query")
        .arg(&query)
        .write_stdin(JSON_INPUT)
        .assert()
        .success()
        .stdout(contains(r#"{"test" :123}"#).and(contains(JSON_EXPECTED.trim()).not()));
}

#[test]
#[cfg(feature = "json")]
fn test_fmt_stdin_invalid_query_override_fails() {
    use predicates::str::contains;

    // If the override is honoured, this must be compiled -- and fail. If it is silently
    // dropped in favour of the built-in query, formatting would succeed instead.
    let tmp_dir = TempDir::new().unwrap();
    let query = tmp_dir.path().join("formatting.scm");
    fs::write(&query, "this is not a tree-sitter query").unwrap();

    initialize();
    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg("--language")
        .arg("json")
        .arg("--query")
        .arg(&query)
        .write_stdin(JSON_INPUT)
        .assert()
        .failure()
        .stderr(contains("this is not a tree-sitter query"));
}

#[test]
#[cfg(feature = "json")]
fn test_fmt_stdin_query_fallback() {
    initialize();
    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        // run in topiary-cli/tests directory so that it couldn't find the
        // default TOPIARY_LANGUAGE_DIR
        .current_dir("tests")
        .arg("fmt")
        .arg("--language")
        .arg("json")
        .write_stdin(JSON_INPUT)
        .assert()
        .success()
        .stdout(JSON_EXPECTED);
}

#[test]
#[cfg(all(feature = "json", feature = "toml"))]
fn test_fmt_files() {
    initialize();
    let json = State::new(JSON_INPUT, "json");
    let toml = State::new(TOML_INPUT, "toml");

    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg(json.path())
        .arg(toml.path())
        .assert()
        .success();

    assert_eq!(json.read(), JSON_EXPECTED);
    assert_eq!(toml.read(), TOML_EXPECTED);
}

#[test]
#[cfg(all(feature = "json", feature = "toml"))]
fn test_fmt_files_query_fallback() {
    initialize();
    let json = State::new(JSON_INPUT, "json");
    let toml = State::new(TOML_INPUT, "toml");

    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        // run in topiary-cli/tests directory so that it couldn't find the
        // default TOPIARY_LANGUAGE_DIR
        .current_dir("tests")
        .arg("fmt")
        .arg(json.path())
        .arg(toml.path())
        .assert()
        .success();

    assert_eq!(json.read(), JSON_EXPECTED);
    assert_eq!(toml.read(), TOML_EXPECTED);
}

#[test]
#[cfg(feature = "json")]
fn test_fmt_dir() {
    initialize();
    let json = State::new(JSON_INPUT, "json");

    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg(json.path().parent().unwrap())
        .assert()
        .success();

    assert_eq!(json.read(), JSON_EXPECTED);
}

#[test]
#[cfg(feature = "json")]
fn test_check_stdin_clean() {
    initialize();
    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg("--check")
        .arg("--language")
        .arg("json")
        .write_stdin(JSON_EXPECTED)
        .assert()
        .success();
}

#[test]
#[cfg(feature = "json")]
fn test_check_stdin_dirty() {
    initialize();
    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg("--check")
        .arg("--language")
        .arg("json")
        .write_stdin(JSON_INPUT)
        .assert()
        .failure();
}

#[test]
#[cfg(feature = "json")]
fn test_check_file_dirty_no_modify() {
    initialize();
    let json = State::new(JSON_INPUT, "json");
    let original_content = json.read();

    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg("--check")
        .arg(json.path())
        .assert()
        .failure();

    // The file must NOT be modified by --check
    assert_eq!(json.read(), original_content);
}

#[test]
#[cfg(feature = "json")]
fn test_check_file_clean() {
    initialize();
    let json = State::new(JSON_EXPECTED, "json");

    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg("--check")
        .arg(json.path())
        .assert()
        .success();
}

#[test]
#[cfg(feature = "json")]
fn test_fmt_invalid() {
    initialize();
    let mut topiary = cargo_bin_cmd!("topiary");

    // Can't specify --language with input files
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg("--language")
        .arg("json")
        .arg("/path/to/some/input")
        .assert()
        .failure();

    // Can't specify --query without --language
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../whatever")
        .arg("fmt")
        .arg("--query")
        .arg("/path/to/query")
        .assert()
        .failure();
}

#[test]
#[cfg(all(feature = "ocamllex", feature = "ocaml"))]
fn test_fmt_ocamllex_invalid_inner_ocaml_fails() {
    use predicates::str::contains;

    let input = fs::read_to_string("tests/samples/input/ocamllex_invalid_inner.mll").unwrap();

    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg("--language")
        .arg("ocamllex")
        .write_stdin(input)
        .assert()
        .failure()
        .stderr(contains("Parsing"));
}

#[test]
#[cfg(all(feature = "ocamllex", feature = "ocaml"))]
fn test_fmt_ocamllex_broken_inner_language_resolution_fails() {
    use predicates::{prelude::PredicateBooleanExt, str::contains};

    let tmp_dir = TempDir::new().unwrap();
    let ocaml_dir = tmp_dir.path().join("ocaml");
    fs::create_dir_all(&ocaml_dir).unwrap();
    fs::write(
        ocaml_dir.join("formatting.scm"),
        "this is not a tree-sitter query",
    )
    .unwrap();

    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        .env("TOPIARY_LANGUAGE_DIR", tmp_dir.path())
        .arg("fmt")
        .arg("--language")
        .arg("ocamllex")
        .write_stdin(r#"rule token = parse | "x" { let values=[1;2;3] in values }"#)
        .assert()
        .failure()
        .stderr(
            contains(r#"Could not resolve injected language "ocaml""#)
                .and(contains("Query error"))
                .and(contains(format!(
                    "ocaml{}formatting.scm",
                    std::path::MAIN_SEPARATOR
                )))
                .and(contains("this is not a tree-sitter query")),
        );
}

#[test]
#[cfg(feature = "json")]
fn test_vis() {
    use predicates::{
        prelude::PredicateBooleanExt,
        str::{ends_with, starts_with},
    };

    initialize();
    let mut topiary = cargo_bin_cmd!("topiary");

    // Sanity check output is a valid DOT graph
    let is_graph = starts_with("graph {").and(ends_with("}\n"));

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("vis")
        .arg("--language")
        .arg("json")
        .write_stdin(JSON_INPUT)
        .assert()
        .success()
        .stdout(is_graph);
}

#[test]
#[cfg(feature = "json")]
fn test_vis_invalid() {
    initialize();
    let mut topiary = cargo_bin_cmd!("topiary");

    // Can't specify --language with input file
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("vis")
        .arg("--language")
        .arg("json")
        .arg("/path/to/some/input")
        .assert()
        .failure();

    // Can't specify --query without --language
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("vis")
        .arg("--query")
        .arg("/path/to/query")
        .assert()
        .failure();

    // Can't specify multiple input files
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("vis")
        .arg("/path/to/some/input")
        .arg("/path/to/another/input")
        .assert()
        .failure();
}

#[test]
fn test_cfg() {
    let mut topiary = cargo_bin_cmd!("topiary");

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("cfg")
        .assert()
        .success()
        .stdout(IsToml);
}

struct IsToml;

impl predicates::Predicate<str> for IsToml {
    fn eval(&self, variable: &str) -> bool {
        toml::Value::try_from(variable).is_ok()
    }
}

impl predicates::reflection::PredicateReflection for IsToml {}

impl fmt::Display for IsToml {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "is_toml")
    }
}

#[test]
fn test_cfg_field_with_custom_queries() {
    use predicates::str::contains;

    let tmp_dir = TempDir::new().unwrap();

    let formatting_path = "/foo/bar/formatting.scm";
    let injections_path = "/foo/bar/injections.scm";

    let languages_ncl = format!(
        r#"{{
  languages.markdown.queries = {{
    formatting.source.path = "{formatting_path}",
    injections.source.path = "{injections_path}",
  }},
}}
"#
    );

    let config_file = tmp_dir.path().join("languages.ncl");
    let mut f = File::create(&config_file).unwrap();
    f.write_all(languages_ncl.as_bytes()).unwrap();

    cargo_bin_cmd!("topiary")
        .arg("--configuration")
        .arg(&config_file)
        .arg("cfg")
        .arg("--field")
        .arg("languages.markdown.queries.formatting.source.path")
        .assert()
        .success()
        .stdout(contains(formatting_path));

    cargo_bin_cmd!("topiary")
        .arg("--configuration")
        .arg(&config_file)
        .arg("cfg")
        .arg("--field")
        .arg("languages.markdown.queries.injections.source.path")
        .assert()
        .success()
        .stdout(contains(injections_path));
}
