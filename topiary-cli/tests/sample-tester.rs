use assert_cmd::cargo_bin_cmd;
use std::fs;
use std::path::PathBuf;
use topiary_core::test_utils::pretty_assert_eq;

use tempfile::TempDir;

fn get_file_extension(language: &str) -> &str {
    match language {
        "bash" => "sh",
        "css" => "css",
        "json" => "json",
        "json5" => "json5",
        "nickel" => "ncl",
        "ocaml" => "ml",
        "ocaml_interface" => "mli",
        "ocamllex" => "mll",
        "openscad" => "scad",
        "rust" => "rs",
        "sdml" => "sdml",
        "toml" => "toml",
        "tree_sitter_query" => "scm",
        "wit" => "wit",
        _ => panic!("Invalid language input: {language}"),
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! lang_test {
    ($($lang:literal,)+ $test_fn:ident) => {
        pastey::paste! {$(
            #[cfg(feature = $lang)]
            #[test]
            fn [<$test_fn _ $lang>]() {
                $test_fn($lang);
            }
        )+}
    };
}

#[cfg(test)]
mod test_fmt {
    use super::*;

    #[allow(unused)]
    fn fmt_input(lang: &str) {
        let file = format!("{lang}.{}", get_file_extension(lang));
        let input = PathBuf::from(format!("tests/samples/input/{file}"));
        let expected = PathBuf::from(format!("tests/samples/expected/{file}"));

        // Make sure our test makes sense
        assert!(input.exists() && expected.exists());

        // Load the known good formatted file
        let expected_output = fs::read_to_string(&expected).unwrap();

        // Stage the input to a temporary directory
        let tmp = TempDir::new().unwrap();
        let staged = tmp.path().join(file);
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

    lang_test!(
        "bash",
        "css",
        "json",
        "json5",
        "nickel",
        "ocaml",
        "ocaml_interface",
        "ocamllex",
        "openscad",
        "rust",
        "sdml",
        "toml",
        "tree_sitter_query",
        "wit",
        fmt_input
    );

    // Test that our query files are properly formatted
    #[test]
    #[cfg(feature = "tree_sitter_query")]
    fn fmt_queries() {
        let language_dir = fs::read_dir("../topiary-queries/queries").unwrap();

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

#[cfg(test)]
mod test_coverage {
    use super::*;

    #[allow(unused)]
    fn coverage_input(lang: &str) {
        let file = format!("{lang}.{}", get_file_extension(lang));
        let input = PathBuf::from(format!("tests/samples/input/{file}"));

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
            panic!("Insufficient coverage of \"{file}\"")
        }
    }

    lang_test!(
        "bash",
        "css",
        "json",
        "json5",
        "nickel",
        "ocaml",
        // "ocaml_interface.mli" is voluntarily omitted:
        // the queries should all be covered by "ocaml.ml"
        // "ocaml_interface",
        "ocamllex",
        "openscad",
        "rust",
        "sdml",
        "toml",
        "tree_sitter_query",
        "wit",
        coverage_input
    );
}

// Test that our query files are properly formatted
#[test]
#[cfg(feature = "tree_sitter_query")]
fn formatted_query_tester() {
    let language_dir = fs::read_dir("../topiary-queries/queries").unwrap();

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
