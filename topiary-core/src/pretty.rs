//! After being split into Atoms, and the queries having been applied this
//! module is responsible for rendering the slice of Atoms back into a displayable
//! format.

use std::{cmp::Ordering, fmt::Write};

use rootcause::prelude::ResultExt;

use crate::{Atom, Capitalisation, FormatterError, FormatterResult};

/// Renders a slice of [`Atom`]s into formatted source code.
///
/// This is the final stage of the formatting pipeline. It walks through the
/// atom list produced by [`apply_query_tree`](crate::tree_sitter::apply_query_tree),
/// interpreting each atom to emit text, newlines, and indentation into the
/// output buffer.
///
/// The `indent` parameter specifies the string used for one level of
/// indentation (e.g. `"  "`, `"    "`, or `"\t"`).
///
/// # Errors
///
/// Returns an error if an atom that should have been removed during
/// post-processing is still present, or if indentation blocks are
/// mismatched.
pub fn render(atoms: &[Atom], indent: &str) -> FormatterResult<String> {
    let mut buffer = String::new();
    let mut indent_level: usize = 0;

    for atom in atoms {
        match atom {
            Atom::Blankline => {
                write!(buffer, "\n\n{}", indent.repeat(indent_level)).context_to()?
            }

            Atom::Empty => (),

            Atom::Hardline => write!(buffer, "\n{}", indent.repeat(indent_level)).context_to()?,

            Atom::IndentEnd => {
                if indent_level == 0 {
                    rootcause::bail!(FormatterError::Query(
                        "Trying to close an unopened indentation block".to_owned(),
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
                rootcause::bail!(FormatterError::Internal(format!(
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

    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        result.push(c);

        if c == '\n' && !matches!(chars.peek(), Some('\n') | None) {
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

#[test]
fn test0() -> Result<(), ()> {
    // let content = "";
    // let content = " \n a";
    let content = "\t\n   a\n   b\n     c\n ";
    let whitespace_prefixes = content
        .split("\n")
        .map(|s| s.strip_suffix("\r").unwrap_or(s)) // to do. remove this?
        .map(str::chars)
        .map(|s| s.take_while(|c| c.is_whitespace()));
    let common_whitespace_prefix = common_prefix_all(whitespace_prefixes.clone()).ok_or(())?;
    let common_whitespace_prefix_len_chars = common_whitespace_prefix.count();
    // let common_whitespace_prefix_len_utf8: usize = common_whitespace_prefix.map(char::len_utf8).sum();
    let minimum_whitespace_prefix_len = whitespace_prefixes.map(Iterator::count).min().ok_or(())?;
    match common_whitespace_prefix_len_chars.cmp(&minimum_whitespace_prefix_len) {
        Ordering::Less => println!("warning"), // to do
        Ordering::Equal => (),
        Ordering::Greater => panic!(
            "the common whitespace prefix should be a substring of the shortest whitespace prefix."
        ),
    }
    Ok(())
}

#[test]
fn test_common_prefix_len_all() -> Result<(), ()> {
    assert_eq!(
        common_prefix_all(["012a", "01b", "0123c"].into_iter().map(str::chars))
            .ok_or(())?
            .collect::<Vec<_>>(),
        vec!['0', '1']
    );
    Ok(())
}

#[test]
fn test_common_whitespace_prefix_len_all0() {
    assert_eq!(
        common_whitespace_prefix_len(["012a", "01b", "0123c"]),
        Some(0)
    );
}

#[test]
fn test_common_whitespace_prefix_len_all1() {
    assert_eq!(
        common_whitespace_prefix_len(["   a", "  b", "    c"]),
        Some(2)
    );
}

fn common_whitespace_prefix_len<'a, SS>(list_of_strings: SS) -> Option<usize>
where
    SS: IntoIterator<Item = &'a str>,
    SS::IntoIter: Clone,
{
    Some(
        common_prefix_all(
            list_of_strings
                .into_iter()
                .map(str::chars)
                .map(|s| s.take_while(|c| c.is_whitespace())),
        )?
        .map(char::len_utf8)
        .sum(),
    )
}

fn common_prefix_all<'a, TSS>(
    list_of_lists: TSS,
) -> Option<impl Iterator<Item = <TSS::Item as IntoIterator>::Item>>
where
    TSS: IntoIterator + 'a,
    TSS::Item: IntoIterator,
    <TSS::Item as IntoIterator>::Item: PartialEq,
{
    list_of_lists.into_iter().fold(None, |accumulator, list| {
        Some(match accumulator {
            None => Box::new(list.into_iter()) as Box<dyn Iterator<Item = _>>,
            Some(a) => Box::new(common_prefix(a, list)),
        })
    })
}

trait CloneableIterator: Iterator + Clone {
    type Item;
}

impl<T: Iterator + Clone> CloneableIterator for T {
    type Item = <T as Iterator>::Item;
}

fn common_prefix<T: PartialEq>(
    list0: impl IntoIterator<Item = T>,
    list1: impl IntoIterator<Item = T>,
) -> impl Iterator<Item = T> {
    list0
        .into_iter()
        .zip(list1.into_iter())
        .take_while(|(element0, element1)| element0 == element1)
        .map(|(a, _)| a)
}
