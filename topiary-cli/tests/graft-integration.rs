use std::{collections::HashSet, env, fs, path::Path, process::Command as ProcessCommand};

use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;

const JSON_INPUT: &str = r#"{   "test"  :123}"#;
const JSON_EXPECTED: &str = "{ \"test\": 123 }\n";

fn configured_path(variable: &str) -> Option<std::path::PathBuf> {
    env::var_os(variable).map(Into::into)
}

fn assert_formats(source: &str, cache: &Path) {
    let fixture = tempfile::tempdir().unwrap();
    let config = fixture.path().join("languages.ncl");
    fs::write(
        &config,
        format!(
            r#"{{
  languages.json = {{
    extensions = ["json"],
    grammar = {{ source = {source} }},
  }},
}}"#
        ),
    )
    .unwrap();

    let mut command = cargo_bin_cmd!("topiary");
    command
        .env("XDG_CACHE_HOME", cache)
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("--configuration")
        .arg(config)
        .arg("fmt")
        .arg("--language")
        .arg("json")
        .write_stdin(JSON_INPUT)
        .assert()
        .success()
        .stdout(predicate::eq(JSON_EXPECTED));
}

#[test]
fn formats_with_an_extensionless_nixpkgs_library() {
    let Some(library) = configured_path("TOPIARY_GRAFT_JSON_LIBRARY") else {
        eprintln!("skipping: TOPIARY_GRAFT_JSON_LIBRARY is supplied by the Nix test environment");
        return;
    };
    assert_eq!(library.file_name().unwrap(), "parser");
    assert!(library.extension().is_none());

    let cache = tempfile::tempdir().unwrap();
    assert_formats(
        &format!(
            r#"{{ library_path = {{ path = "{}" }} }}"#,
            library.display()
        ),
        cache.path(),
    );
}

#[test]
fn preserves_graft_symbol_error_context() {
    let Some(library) = configured_path("TOPIARY_GRAFT_JSON_LIBRARY") else {
        eprintln!("skipping: TOPIARY_GRAFT_JSON_LIBRARY is supplied by the Nix test environment");
        return;
    };
    let cache = tempfile::tempdir().unwrap();
    let loader = topiary_config::language::GrammarLoader::new(cache.path()).unwrap();
    let language = topiary_config::language::Language::new(
        "json".to_owned(),
        topiary_config::language::LanguageConfiguration {
            extensions: HashSet::from(["json".to_owned()]),
            indent: None,
            grammar: topiary_config::language::Grammar::new(
                topiary_config::language::GrammarSource::library_path(library),
            )
            .with_symbol("tree_sitter_definitely_missing"),
            queries: None,
        },
    );

    let error = language.grammar_with(&loader).unwrap_err().to_string();
    assert!(error.contains("json"));
    assert!(error.contains("tree_sitter_definitely_missing"));
}

#[test]
fn formats_with_a_read_only_nixpkgs_source() {
    let Some(source) = configured_path("TOPIARY_GRAFT_JSON_SOURCE") else {
        eprintln!("skipping: TOPIARY_GRAFT_JSON_SOURCE is supplied by the Nix test environment");
        return;
    };

    let cache = tempfile::tempdir().unwrap();
    assert_formats(
        &format!(r#"{{ source_path = {{ path = "{}" }} }}"#, source.display()),
        cache.path(),
    );
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn git(repository: &Path, arguments: &[&str]) -> String {
    let output = ProcessCommand::new("git")
        .args(arguments)
        .current_dir(repository)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("HOME", repository)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

#[test]
fn shares_a_local_git_checkout_between_grammar_and_query() {
    let Some(source) = configured_path("TOPIARY_GRAFT_JSON_SOURCE") else {
        eprintln!("skipping: TOPIARY_GRAFT_JSON_SOURCE is supplied by the Nix test environment");
        return;
    };

    let fixture = tempfile::tempdir().unwrap();
    let repository = fixture.path().join("repository");
    copy_tree(&source, &repository);
    fs::write(
        repository.join("formatting.scm"),
        include_str!("../../topiary-queries/queries/json/formatting.scm"),
    )
    .unwrap();
    git(&repository, &["init", "--quiet"]);
    git(&repository, &["config", "user.name", "Topiary test"]);
    git(
        &repository,
        &["config", "user.email", "topiary-test@example.invalid"],
    );
    git(&repository, &["add", "."]);
    git(&repository, &["commit", "--quiet", "-m", "JSON fixture"]);
    let commit = git(&repository, &["rev-parse", "HEAD"]);

    let config = fixture.path().join("languages.ncl");
    let url = format!("file://{}", repository.display());
    fs::write(
        &config,
        format!(
            r#"{{
  languages.json = {{
    extensions = ["json"],
    grammar.source.git = {{ url = "{url}", rev = "{commit}" }},
    queries.formatting.source = {{
      git = {{ url = "{url}", rev = "{commit}" }},
      path = "formatting.scm",
    }},
  }},
}}"#
        ),
    )
    .unwrap();

    let cache = fixture.path().join("cache");
    let mut command = cargo_bin_cmd!("topiary");
    command
        .env("XDG_CACHE_HOME", &cache)
        .arg("--configuration")
        .arg(config)
        .arg("fmt")
        .arg("--language")
        .arg("json")
        .write_stdin(JSON_INPUT)
        .assert()
        .success()
        .stdout(predicate::eq(JSON_EXPECTED));

    let checkouts = cache.join("topiary/tree-sitter-graft/sources/checkouts");
    assert_eq!(fs::read_dir(checkouts).unwrap().count(), 1);
}
