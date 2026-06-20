//! After being split into Atoms, and the queries having been applied this
//! module is responsible for rendering the slice of Atoms back into a displayable
//! format.

use std::{cmp::Ordering, fmt::Write};

use rootcause::prelude::ResultExt;

use crate::{
    AbsoluteIndentation, Atom, Capitalisation, FormatterError, FormatterResult, MultiLineIndent,
};

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
                    MultiLineIndent::AbsoluteIndentation(absolute_indentation) => {
                        render_absolute_indentation(
                            absolute_indentation,
                            content,
                            indent_level,
                            indent,
                        )
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

/// formats multi line source code constructs like multi line strings.
///
/// `absolute_indentation` contains configuration. at this stage we assume that it is a `ClosingColumnInsignificant` constructor.
/// `content_input` is the source code inbetween the delimiters of the multi line construct like `''` in the example of nix multi line strings or `"""` in the example of c# multi line strings.
/// the returned `String` differs only in white space from `content_input`.
fn render_absolute_indentation(
    absolute_indentation: AbsoluteIndentation,
    content_input: &str,
    indent_level: usize,
    indent: &str,
) -> String {
    let AbsoluteIndentation::ClosingColumnInsignificant {
        last_line_break_significant,
    } = absolute_indentation
    else {
        todo!()
    };

    let content_collected: Vec<&str> = content_input
        .split("\n")
        .map(|s| s.strip_suffix("\r").unwrap_or(s)) // to do. remove this?
        .collect(); // because we need `DoubleEndedIterator`.

    if content_collected.len() == 1 {
        return content_input.to_owned();
    }

    let mut content = content_collected.iter().copied();
    let mut buffer = String::new();

    let first_line = content
        .clone()
        .next()
        .expect("`split` should not produce empty iterators.");
    if first_line.chars().all(char::is_whitespace) {
        content
            .next()
            .expect("`split` should not produce empty iterators.");
    }

    let last_line_is_whitespace = content
        .clone()
        .next_back()
        .expect("`split` should not produce empty iterators and `content_collected.len() != 1`.")
        .chars()
        .all(char::is_whitespace);
    if last_line_is_whitespace {
        content.next_back().expect(
            "`split` should not produce empty iterators and `content_collected.len() != 1`.",
        );
    }

    if content.clone().next().is_none() {
        return "".to_owned();
    }

    let whitespace_prefixes = content
        .clone()
        .filter(|s| s.chars().any(|c| !c.is_whitespace()))
        .map(str::chars)
        .map(|s| s.take_while(|c| c.is_whitespace()));
    let common_whitespace_prefix = common_prefix(whitespace_prefixes.clone())
        .expect("`next().is_none()` should still not hold because it just did for a `clone()`.");
    match common_whitespace_prefix.clone().count().cmp(
        &whitespace_prefixes.map(Iterator::count).min().expect(
            "`next().is_none()` should still not hold because it just did for a `clone()`.",
        ),
    ) {
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

    for line in content {
        if line.chars().all(char::is_whitespace) {
            writeln!(buffer).unwrap();
        } else {
            write!(buffer, "\n{}{}", indent.repeat(indent_level + 1), line).unwrap();
        }
    }
    if last_line_is_whitespace || !last_line_break_significant {
        write!(buffer, "\n{}", indent.repeat(indent_level)).unwrap();
    }
    buffer
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

mod tests {
    #[allow(unused_imports)]
    use super::*;

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

    #[test]
    fn test_render_absolute_indentation0() {
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false
                },
                "\t
    a
   b
     c
            ",
                3,
                "    ",
            ),
            "
                 a
                b
                  c
            ",
        );
    }

    #[test]
    fn test_render_absolute_indentation1() {
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                },
                "x
    a
   b
     c
            ",
                3,
                "    "
            ),
            "
                x
                    a
                   b
                     c
            "
        );
    }

    #[test]
    fn test_render_absolute_indentation_1_line0() {
        for line in ["", " ", " a"] {
            assert_eq!(
                render_absolute_indentation(
                    AbsoluteIndentation::ClosingColumnInsignificant {
                        last_line_break_significant: false,
                    },
                    line,
                    3,
                    "  ",
                ),
                line,
            );
        }
    }

    #[test]
    fn test_render_absolute_indentation_1_line1() {
        for line in ["", " ", " a"] {
            assert_eq!(
                render_absolute_indentation(
                    AbsoluteIndentation::ClosingColumnInsignificant {
                        last_line_break_significant: true,
                    },
                    line,
                    3,
                    "  ",
                ),
                line,
            );
        }
    }

    #[test]
    #[ignore]
    fn test_render_absolute_indentation_1_line2() {
        for line in ["", " ", " a"] {
            assert_eq!(
                render_absolute_indentation(
                    AbsoluteIndentation::ClosingColumnSignificant,
                    line,
                    3,
                    "    ",
                ),
                line,
            );
        }
    }

    #[test]
    #[ignore]
    fn test_render_absolute_indentation_1_line3() {
        for line in ["", " ", " a"] {
            assert_eq!(
                render_absolute_indentation(AbsoluteIndentation::Comment, line, 3, "    "),
                line.trim_end()
            );
        }
    }

    #[test]
    fn test_render_absolute_indentation_2_lines0() {
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                },
                "
",
                3,
                "    ",
            ),
            "",
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                },
                " 
",
                3,
                "    ",
            ),
            "",
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                },
                "    a
",
                3,
                "    ",
            ),
            "
                a
            ",
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                },
                "
    ",
                3,
                "    ",
            ),
            "",
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                },
                " 
    ",
                3,
                "    ",
            ),
            "",
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                },
                "    a
    ",
                3,
                "    ",
            ),
            "
                a
            "
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                },
                "
    a",
                3,
                "    ",
            ),
            "
                a
            "
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                },
                " 
    a",
                3,
                "    ",
            ),
            "
                a
            "
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                },
                "        a
    a",
                3,
                "    ",
            ),
            "
                    a
                a
            "
        );
    }

    #[test]
    fn test_render_absolute_indentation_2_lines1() {
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                },
                "
",
                3,
                "    ",
            ),
            "",
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                },
                " 
",
                3,
                "    ",
            ),
            "",
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                },
                "    a
",
                3,
                "    ",
            ),
            "
                a
            ",
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                },
                "
    ",
                3,
                "    ",
            ),
            "",
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                },
                " 
    ",
                3,
                "    ",
            ),
            "",
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                },
                "    a
    ",
                3,
                "    ",
            ),
            "
                a
            "
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                },
                "
    a",
                3,
                "    ",
            ),
            "
                a"
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                },
                " 
    a",
                3,
                "    ",
            ),
            "
                a"
        );
        assert_eq!(
            render_absolute_indentation(
                AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                },
                "        a
    a",
                3,
                "    ",
            ),
            "
                    a
                a"
        );
    }
}
