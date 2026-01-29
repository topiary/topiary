//! After being split into Atoms, and the queries having been applied this
//! module is responsible for rendering the slice of Atoms back into a displayable
//! format.

use std::fmt::Write;

use rootcause::prelude::ResultExt;

use crate::{Atom, Capitalisation, FormatterError, FormatterResult};

/// Renders a slice of Atoms into an owned string.
/// The indent &str is used when an `Atom::IdentStart` is encountered.
/// Any string is accepted, but you will probably want to specify something
/// along the lines of "  " "    " or "\t".
///
/// # Errors
///
/// If an unexpected Atom is encountered, a `FormatterError::Internal` is returned.
pub fn render(atoms: &[Atom], indent: &str) -> FormatterResult<String> {
    let mut buffer = String::new();
    let mut indent_level: usize = 0;

    for atom in atoms {
        match atom {
            Atom::Blankline => write!(buffer, "\n\n{}", indent.repeat(indent_level))?,

            Atom::Empty => (),

            Atom::Hardline => write!(buffer, "\n{}", indent.repeat(indent_level))?,

            Atom::IndentEnd => {
                if indent_level == 0 {
                    return Err(FormatterError::Query(
                        "Trying to close an unopened indentation block".into(),
                    ));
                }

                indent_level -= 1;
            }

            Atom::IndentStart => indent_level += 1,

            Atom::Leaf {
                content,
                original_position,
                single_line_no_indent,
                multi_line_indent_all,
                keep_whitespace,
                capitalisation,
                ..
            } => {
                if *single_line_no_indent {
                    // The line break after the content has been previously added
                    // as a `Hardline` in the atom stream.
                    writeln!(buffer).context_to()?;
                }
                let content = if *keep_whitespace {
                    content
                } else {
                    content.trim_end_matches('\n')
                };

                let mut content = if *multi_line_indent_all {
                    let cursor = current_column(&buffer) as i32;

                    // original_position is 1-based
                    let original_column = original_position.column as i32 - 1;

                    let indenting = cursor - original_column;

                    // The following assumes spaces are used for indenting
                    match indenting {
                        0 => content.into(),
                        n if n > 0 => add_spaces_after_newlines(content, indenting),
                        _ => try_removing_spaces_after_newlines(content, -indenting),
                    }
                } else {
                    content.into()
                };
                match capitalisation {
                    Capitalisation::UpperCase => {
                        content = content.to_uppercase();
                    }
                    Capitalisation::LowerCase => {
                        content = content.to_lowercase();
                    }
                    _ => {}
                }
                write!(buffer, "{content}").context_to()?;
            }

            Atom::Literal(s) => write!(buffer, "{s}").context_to()?,

            Atom::Space => write!(buffer, " ").context_to()?,

            // All other atom kinds should have been post-processed at that point
            other => {
                return Err(FormatterError::Internal(format!(
                    "Found atom that should have been removed before rendering: {other:?}",
                )));
            }
        };
    }

    Ok(buffer)
}

fn current_column(s: &str) -> usize {
    s.chars().rev().take_while(|c| *c != '\n').count()
}

fn add_spaces_after_newlines(s: &str, n: i32) -> String {
    let mut result = String::new();

    let chars = s.chars().peekable();

    for c in chars {
        result.push(c);

        if c == '\n' {
            for _ in 0..n {
                result.push(' ');
            }
        }
    }

    result
}

fn try_removing_spaces_after_newlines(s: &str, n: i32) -> String {
    let mut result = String::new();

    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        result.push(c);

        if c == '\n' {
            for _ in 0..n {
                if let Some(' ') = chars.peek() {
                    chars.next();
                } else {
                    break;
                }
            }
        }
    }

    result
}
