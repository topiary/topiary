//! After being split into Atoms, and the queries having been applied this
//! module is responsible for rendering the slice of Atoms back into a displayable
//! format.

use std::{cmp::Ordering, fmt::Write};

use rootcause::prelude::ResultExt;

use crate::{Atom, Capitalisation, FormatterError, FormatterResult, MultiLineIndent};

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

                let mut content = match *multi_line_indent_all {
                    MultiLineIndent::None => content.into(),
                    MultiLineIndent::RelativeIndentation => {
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
                    }
                    MultiLineIndent::AbsoluteIndentation => {
                        render_multi_line_string(content, indent_level, indent)
                    }
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
fn test0() {
    let indent_level: usize = 1;
    let indent = "  ";
    // let content = "";
    // let content = " \n a";
    let content = "\t\n   a\n   b\n     c\n ";
    assert_eq!(
        render_multi_line_string(content, indent_level, indent),
        "\n    a\n    b\n      c\n  "
    );
}

fn render_multi_line_string(content: &str, indent_level: usize, indent: &str) -> String {
    let content: Vec<&str> = content
        .split("\n")
        .map(|s| s.strip_suffix("\r").unwrap_or(s)) // to do. remove this?
        .collect(); // because we need `DoubleEndedIterator`.
    let mut content = content.iter().copied();
    debug_assert!(if let Some(s) = content.next() {
        s.chars().all(char::is_whitespace)
    } else {
        false
    });
    debug_assert!(if let Some(s) = content.next_back() {
        s.chars().all(char::is_whitespace)
    } else {
        false
    });
    let whitespace_prefixes = content
        .clone()
        .filter(|s| s.chars().any(|c| !c.is_whitespace()))
        .map(str::chars)
        .map(|s| s.take_while(|c| c.is_whitespace()));
    let common_whitespace_prefix = common_prefix(whitespace_prefixes.clone()).unwrap();
    match common_whitespace_prefix
        .clone()
        .count()
        .cmp(&whitespace_prefixes.map(Iterator::count).min().unwrap())
    {
        Ordering::Less => println!("warning"), // to do
        Ordering::Equal => (),
        Ordering::Greater => panic!(
            "the common whitespace prefix should be a substring of the shortest whitespace prefix."
        ),
    }
    let common_whitespace_prefix_len_utf8: usize =
        common_whitespace_prefix.map(char::len_utf8).sum();
    let content = content.map(|line| {
        if common_whitespace_prefix_len_utf8 < line.len() {
            &line[common_whitespace_prefix_len_utf8..]
        } else {
            ""
        }
    });

    let mut buffer = String::new();
    for line in content {
        if line.chars().all(char::is_whitespace) {
            write!(buffer, "\n").unwrap();
        } else {
            write!(buffer, "\n{}{}", indent.repeat(indent_level + 1), line).unwrap();
        }
    }
    write!(buffer, "\n{}", indent.repeat(indent_level)).unwrap();
    buffer
}

#[test]
fn test_common_prefix() {
    assert_eq!(
        common_prefix(["012a", "01b", "0123c"].map(str::chars))
            .map(Iterator::collect::<String>)
            .as_ref()
            .map(String::as_str),
        Some("01")
    );
}

fn common_prefix<TSS>(
    list_of_lists: TSS,
) -> Option<impl Iterator<Item = <TSS::Item as IntoIterator>::Item> + Clone>
where
    TSS: IntoIterator,
    TSS::Item: IntoIterator,
    <TSS::Item as IntoIterator>::IntoIter: Clone,
    <TSS::Item as IntoIterator>::Item: PartialEq,
{
    let mut iters = list_of_lists
        .into_iter()
        .map(IntoIterator::into_iter)
        .collect::<Vec<_>>();
    if iters.is_empty() {
        return None;
    }
    Some(std::iter::from_fn(move || {
        let mut items = iters.iter_mut().map(Iterator::next);
        let first = items.next().expect("`iters` should not be empty.")?;
        items
            .all(|item| item.as_ref() == Some(&first))
            .then_some(first)
    }))
}
