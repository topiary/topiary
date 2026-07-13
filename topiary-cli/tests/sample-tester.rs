use assert_cmd::cargo_bin_cmd;
use std::fs;
use std::path::PathBuf;
use topiary_core::test_utils::pretty_assert_eq;

use tempfile::TempDir;

#[allow(unused)]
pub(crate) fn fmt_input(lang: &str, filename: &str) {
    let input = PathBuf::from(format!("tests/samples/input/{lang}/{filename}"));
    let expected = PathBuf::from(format!("tests/samples/expected/{lang}/{filename}"));

    // Make sure our test makes sense
    assert!(input.exists() && expected.exists());

    // Load the known good formatted file
    let expected_output = fs::read_to_string(&expected).unwrap();

    // Stage the input to a temporary directory
    let tmp = TempDir::new().unwrap();
    let staged = tmp.path().join(filename);
    fs::copy(input, &staged).unwrap();

    // Run Topiary against the staged input file
    let mut topiary = cargo_bin_cmd!("topiary");
    let output = topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries/")
        .arg("fmt")
        .arg(&staged)
        .output()
        .expect("Failed to run `topiary fmt`");

    // Print the invocation output, so that it can be inspected with --nocapture
    let output_str = String::from_utf8(output.stderr).expect("Failed to decode Topiary output");
    println!("{output_str}");

    // Read the file after formatting
    let formatted = fs::read_to_string(&staged).unwrap();

    // Assert the formatted file is as expected
    pretty_assert_eq(&expected_output, &formatted);
}

#[allow(unused)]
pub(crate) fn check_input(lang: &str, filename: &str) {
    let input = PathBuf::from(format!("tests/samples/input/{lang}/{filename}"));
    let expected = PathBuf::from(format!("tests/samples/expected/{lang}/{filename}"));

    // Make sure our test makes sense
    assert!(input.exists() && expected.exists());

    // The input file is unformatted, so --check should fail
    let mut topiary = cargo_bin_cmd!("topiary");
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries/")
        .arg("fmt")
        .arg("--check")
        .arg(&input)
        .assert()
        .failure();

    // The expected file is already formatted, so --check should succeed
    let mut topiary = cargo_bin_cmd!("topiary");
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries/")
        .arg("fmt")
        .arg("--check")
        .arg(&expected)
        .assert()
        .success();
}

#[allow(unused)]
pub(crate) fn coverage_input(lang: &str, filename: &str) {
    let input = PathBuf::from(format!("tests/samples/input/{lang}/{filename}"));

    // Make sure our test makes sense
    assert!(input.exists());

    // Run `topiary coverage` against the input file
    let mut topiary = cargo_bin_cmd!("topiary");
    let output = topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries/")
        .arg("coverage")
        .arg(&input)
        .output()
        .expect("Failed to run `topiary coverage`");

    if !output.status.success() {
        panic!("Insufficient coverage of \"{input:?}\"")
    }
}

// Test that our query files are properly formatted
#[test]
#[cfg(feature = "tree_sitter_query")]
fn formatted_query_tester() {
    // Top level query directory
    let query_dir = fs::read_dir("../topiary-queries/queries").unwrap();

    for language_dir in query_dir {
        let language_dir = fs::read_dir(language_dir.unwrap().path()).unwrap();
        for file in language_dir {
            let file = file.unwrap();

            // Load the query file (we assume is formatted correctly)
            let expected = fs::read_to_string(file.path()).unwrap();

            let tmp_dir = TempDir::new().unwrap();

            // Copy the file to a temp dir
            let mut input_file = tmp_dir.path().to_path_buf();
            input_file.push(file.path().file_name().unwrap());
            fs::copy(file.path(), &input_file).unwrap();

            // Run topiary on the input file in the temp dir
            let mut topiary = cargo_bin_cmd!("topiary");
            topiary
                .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries/")
                .arg("fmt")
                .arg(&input_file)
                .assert()
                .success();

            // Read the file after formatting
            let formatted = fs::read_to_string(input_file).unwrap();

            pretty_assert_eq(&expected, &formatted);
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/generated_sample_tests.rs"));
