use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_sample_tests.rs");

    // Watch the samples directory for changes
    println!("cargo:rerun-if-changed=tests/samples/");

    let mut generated_code = String::new();

    let input_dir = Path::new("tests/samples/input");
    if input_dir.exists() {
        for entry in fs::read_dir(input_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                let lang = path.file_name().unwrap().to_str().unwrap();

                for file_entry in fs::read_dir(&path).unwrap() {
                    let file_entry = file_entry.unwrap();
                    let file_path = file_entry.path();

                    if file_path.is_file() {
                        let filename = file_path.file_name().unwrap().to_str().unwrap();
                        let test_name_suffix = filename.replace(".", "_").replace("-", "_");

                        // Check if expected file exists (some files like ocamllex_invalid_inner.mll are input only for other tests)
                        let expected_path = Path::new("tests/samples/expected")
                            .join(lang)
                            .join(filename);
                        if expected_path.exists() {
                            // Generate fmt and check tests
                            generated_code.push_str(&format!(
                                r#"
#[cfg(feature = "{lang}")]
#[test]
fn test_fmt_{lang}_{test_name_suffix}() {{
    fmt_input("{lang}", "{filename}");
}}

#[cfg(feature = "{lang}")]
#[test]
fn test_check_{lang}_{test_name_suffix}() {{
    check_input("{lang}", "{filename}");
}}
"#
                            ));

                            // Coverage test (exclude ocaml_interface like before)
                            if lang != "ocaml_interface" {
                                // Coverage only needs input file
                                generated_code.push_str(&format!(
                                    r#"
#[cfg(feature = "{lang}")]
#[test]
fn test_coverage_{lang}_{test_name_suffix}() {{
    coverage_input("{lang}", "{filename}");
}}
"#
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    fs::write(&dest_path, generated_code).unwrap();
}
