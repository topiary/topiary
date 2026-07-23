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

                let mut content = match multi_line_indent_all {
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
                        )?
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
/// `content_original` is the source code inbetween the delimiters of the multi line construct like `''` in the example of nix multi line strings or `"""` in the example of c# multi line strings.
/// the returned `String` differs only in white space from `content_original`.
fn render_absolute_indentation(
    absolute_indentation: &AbsoluteIndentation,
    content_original: &str,
    indent_level: usize,
    indent: &str,
) -> FormatterResult<String> {
    let AbsoluteIndentation::ClosingColumnInsignificant {
        last_line_break_significant,
        start,
        end,
    } = absolute_indentation
    else {
        todo!()
    };

    let content: Vec<&str> = content_original
        .strip_prefix(start)
        .ok_or_else(|| {
            FormatterError::Query(format!(
                "the multi line leaf starting with {:?} should start with {start:?} as marked by the query",
                &content_original[..content_original.len().min(16)]
            ))
        })?
        .strip_suffix(end)
        .ok_or_else(|| {
            FormatterError::Query(format!(
                "the multi line leaf ending with {:?} should end with {end:?} as marked by the query",
                &content_original[content_original.len().saturating_sub(16)..]
            ))
        })?
        .split("\n")
        .map(|s| s.strip_suffix("\r").unwrap_or(s)) // to do. remove this?
        .collect(); // because we need `DoubleEndedIterator::next_back`.

    if content.len() == 1 {
        return Ok(content_original.to_owned());
    }

    let mut content = content.iter().copied();
    let mut buffer = String::new();
    write!(buffer, "{start}").unwrap();

    // skip potential empty first line
    if content
        .clone()
        .next()
        .expect("`split` should not produce empty iterators.")
        .chars()
        .all(char::is_whitespace)
    {
        content
            .next()
            .expect("`split` should not produce empty iterators.");
    }

    // skip potential empty last line
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
        return Ok(format!("{start}{end}"));
    }

    let whitespace_prefixes = content
        .clone()
        .filter(|s| !s.chars().all(char::is_whitespace))
        .map(str::chars)
        .map(|s| s.take_while(|c| c.is_whitespace()));
    if let Some(common_whitespace_prefix) = common_prefix(whitespace_prefixes.clone()) {
        // purely for warning generation
        match common_whitespace_prefix.clone().count().cmp(
            &whitespace_prefixes.map(Iterator::count).min().expect(
                "whitespace_prefixes should not be empty if `common_prefix` returns `Some`.",
            ),
        ) {
            Ordering::Less => println!(
                // to do
                "is this indentation? then you should not mix different kinds of whitespace characters."
            ),
            Ordering::Equal => (),
            Ordering::Greater => panic!(
                "the common whitespace prefix should be a substring of the shortest whitespace prefix."
            ),
        }

        let common_whitespace_prefix_len_utf8: usize =
            common_whitespace_prefix.map(char::len_utf8).sum();
        let content =
            content.map(|line| &line[line.len().min(common_whitespace_prefix_len_utf8)..]);

        for line in content {
            if line.is_empty() {
                writeln!(buffer).unwrap();
            } else {
                write!(buffer, "\n{}{line}", indent.repeat(indent_level + 1)).unwrap();
            }
        }
    } else {
        // no lines other than whitespace lines
        for _ in content {
            writeln!(buffer).unwrap();
        }
    }
    if !last_line_break_significant || last_line_is_whitespace {
        write!(buffer, "\n{}", indent.repeat(indent_level)).unwrap();
    }
    write!(buffer, "{end}").unwrap();
    Ok(buffer)
}

/// returns an iterator over the longest common prefix shared by all the
/// inner iterables of `list_of_lists`, or `none` if `list_of_lists` is empty.
///
/// the prefix is computed element-wise: items at the same position in each
/// inner iterable are compared, and the iteration stops as soon as any pair
/// differs (or one of the inner iterables is exhausted).
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
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''\t
    a
   b
     c
            ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                 a
                b
                  c
            ''",
        );
    }

    #[test]
    fn test_render_absolute_indentation1() {
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''x
    a
   b
     c
            ''",
                3,
                "    "
            )
            .unwrap(),
            "''
                x
                    a
                   b
                     c
            ''"
        );
    }

    #[test]
    fn test_render_absolute_indentation_1_line0() {
        for line in ["''''", "'' ''", "'' a''"] {
            assert_eq!(
                render_absolute_indentation(
                    &AbsoluteIndentation::ClosingColumnInsignificant {
                        last_line_break_significant: false,
                        start: "''".to_owned(),
                        end: "''".to_owned(),
                    },
                    line,
                    3,
                    "  ",
                )
                .unwrap(),
                line,
            );
        }
    }

    #[test]
    fn test_render_absolute_indentation_1_line1() {
        for line in ["''''", "'' ''", "'' a''"] {
            assert_eq!(
                render_absolute_indentation(
                    &AbsoluteIndentation::ClosingColumnInsignificant {
                        last_line_break_significant: true,
                        start: "''".to_owned(),
                        end: "''".to_owned(),
                    },
                    line,
                    3,
                    "  ",
                )
                .unwrap(),
                line,
            );
        }
    }

    #[test]
    fn test_render_absolute_indentation_2_lines0() {
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
''",
                3,
                "    ",
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "'' 
''",
                3,
                "    ",
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    a
''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "'' 
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    a
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''"
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''"
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "'' 
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''"
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''        a
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a
                a
            ''"
        );
    }

    #[test]
    fn test_render_absolute_indentation_2_lines1() {
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
''",
                3,
                "    ",
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "'' 
''",
                3,
                "    ",
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    a
''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "'' 
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    a
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''"
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a''"
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "'' 
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a''"
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''        a
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a
                a''"
        );
    }

    #[test]
    fn test_render_absolute_indentation_3_lines0() {
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''

''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    

''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    a

''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    
''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    
    
''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    a
    
''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    a
''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    
    a
''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''        a
    a
''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''

    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    

    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    a

    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    
    
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    a
    
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    a
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    
    a
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''        a
    a
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''

    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''

                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    

    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''

                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''        a

    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a

                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''

                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    
    
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''

                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''        a
    
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a

                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
        a
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    
        a
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''            a
        a
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                        a
                    a
                a
            ''",
        );
    }

    #[test]
    fn test_render_absolute_indentation_3_lines1() {
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''

''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    

''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    a

''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    
''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    
    
''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    a
    
''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    a
''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    
    a
''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''        a
    a
''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''

    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    

    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    a

    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    
    
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    a
    
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    a
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    
    a
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''        a
    a
    ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a
                a
            ''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''

    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''

                a''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    

    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''

                a''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''        a

    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a

                a''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
    
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''

                a''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    
    
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''

                a''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''        a
    
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a

                a''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
        a
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a
                a''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''    
        a
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                    a
                a''",
        );
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''            a
        a
    a''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                        a
                    a
                a''",
        );
    }

    #[test]
    fn test_render_absolute_indentation_significant_all_whitespace() {
        assert_eq!(
            render_absolute_indentation(
                &AbsoluteIndentation::ClosingColumnInsignificant {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
                    a
                     
                ''",
                3,
                "    ",
            )
            .unwrap(),
            "''
                a
                 
            ''",
        );
    }
}
