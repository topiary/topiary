use std::io::BufReader;

use topiary_core::{Language, LanguageResolver, Operation, formatter};

use crate::{
    error::{CLIError, CLIResult, TopiaryError},
    io::{InputFile, read_input},
};

/// Run the formatter on an input and compare the result to the original.
/// Returns `Ok(())` if the input is already formatted, or a `CheckFailed` error
/// containing the original and formatted strings if it is not.
pub fn check_input(
    input: InputFile,
    language: &Language,
    skip_idempotence: bool,
    tolerate_parsing_errors: bool,
    resolve: Option<&LanguageResolver<'_>>,
) -> CLIResult<()> {
    let source_name = input.source().to_string();

    let mut buf_input = BufReader::new(input);
    let original = read_input(&mut buf_input)?;

    let mut formatted_bytes: Vec<u8> = Vec::new();
    formatter(
        &mut original.as_bytes(),
        &mut formatted_bytes,
        language,
        Operation::Format {
            skip_idempotence,
            tolerate_parsing_errors,
        },
        resolve,
    )?;

    let formatted = String::from_utf8_lossy(&formatted_bytes).into_owned();

    if original != formatted {
        return Err(TopiaryError::Bin(
            format!("{source_name} is not formatted"),
            Some(CLIError::CheckFailed {
                source_name,
                original,
                formatted,
            }),
        ));
    }

    Ok(())
}
