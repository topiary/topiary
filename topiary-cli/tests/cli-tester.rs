use std::{fmt, sync::Once};
#[cfg(any(feature = "json", feature = "toml"))]
use {
    std::{fs, fs::File, io::Write, path::PathBuf},
    tempfile::TempDir,
};

use assert_cmd::cargo_bin_cmd;

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
        .arg("../topiary-queries/queries/json.scm")
        .write_stdin(JSON_INPUT)
        .assert()
        .success()
        .stdout(JSON_EXPECTED);
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
