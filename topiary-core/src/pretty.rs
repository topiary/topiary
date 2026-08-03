//! After being split into Atoms, and the queries having been applied this
//! module is responsible for rendering the slice of Atoms back into a displayable
//! format.

use std::{cmp::Ordering, fmt::Write};

use rootcause::prelude::ResultExt;

use crate::{
    Atom, Capitalisation, EnforceIndentation, FormatterError, FormatterResult, MultiLineIndent,
    tree_sitter::Position,
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
                    MultiLineIndent::MaintainOffset => {
                        let cursor = current_column(&buffer) as i32;

                        // original_position is 1-based
                        let original_column = original_position.column as i32 - 1;

                        let indenting = cursor - original_column;

                        // The following assumes spaces are used for indenting
                        match indenting {
                            ..0 => try_removing_spaces_after_newlines(content, -indenting),
                            0 => content.into(),
                            1.. => add_spaces_after_newlines(content, indenting),
                        }
                    }
                    MultiLineIndent::EnforceIndentation(absolute_indentation) => {
                        render_enforced_indentation(
                            absolute_indentation,
                            content,
                            indent_level,
                            indent,
                            original_position,
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
/// `content_original` is the multi line string including the delimiters.
/// `indent_level` is the current indentation level of the line containing the start delimiter.
/// `indent` is the white space prefix of a line at indentation level 1.
/// the returned `String` differs only in white space from `content_original`.
fn render_enforced_indentation(
    absolute_indentation: &EnforceIndentation,
    content_original: &str,
    indent_level: usize,
    indent: &str,
    start_position: &Position,
) -> FormatterResult<String> {
    let EnforceIndentation {
        last_line_break_significant,
        start,
        end,
    } = absolute_indentation;

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
        .map(|s| s.strip_suffix("\r").unwrap_or(s))
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

    let mut common_whitespace_prefix_len: usize = 0;
    let mut common_whitespace_prefix_len_utf8 = 0;
    let mut content_collected: Vec<_> = content.clone().map(str::chars).collect();
    loop {
        let mut nexts = content_collected.iter_mut().filter_map(Iterator::next);
        if let Some(first) = nexts.next()
            && first.is_whitespace()
            && nexts.all(|c| c == first)
        {
            common_whitespace_prefix_len += 1;
            common_whitespace_prefix_len_utf8 += first.len_utf8();
        } else {
            break;
        }
    }

    if log::log_enabled!(log::Level::Info) {
        match content
            .clone()
            .filter(|s| !s.chars().all(char::is_whitespace))
            .map(str::chars)
            .map(|s| s.take_while(|c| c.is_whitespace()))
            .map(Iterator::count)
            .min()
            .map(|min_whitespace_prefix_len| {
                common_whitespace_prefix_len.cmp(&min_whitespace_prefix_len)
            }) {
            None => (), // no lines other than whitespace lines
            // do not change the log level without changing it in the if condition 6 lines above.
            Some(Ordering::Less) => log::info!(
                "the multi line string starting at {} mixes different whitespace characters like spaces and tabs in its lines' whitespace prefixes. \
                    is this supposed to be indentation? then you should not mix different whitespace characters.",
                start_position
            ),
            Some(Ordering::Equal) => (),
            Some(Ordering::Greater) => panic!(
                "the common whitespace prefix should be a substring of the shortest whitespace prefix."
            ),
        }
    }

    let content = content.map(|line| &line[line.len().min(common_whitespace_prefix_len_utf8)..]);

    for line in content {
        if line.is_empty() {
            writeln!(buffer).unwrap();
        } else {
            write!(buffer, "\n{}{line}", indent.repeat(indent_level + 1)).unwrap();
        }
    }
    #[allow(clippy::nonminimal_bool)]
    if last_line_is_whitespace || (!last_line_is_whitespace && !last_line_break_significant) {
        write!(buffer, "\n{}", indent.repeat(indent_level)).unwrap();
    }
    write!(buffer, "{end}").unwrap();
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start_position() -> Position {
        Position { row: 1, column: 1 }
    }

    #[test]
    fn test_render_enforced_indentation0() {
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "\
''\t
    a
   b
     c
''",
                3,
                "    ",
                &start_position(),
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
    fn test_render_enforced_indentation1() {
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "\
''x
    a
   b
     c
''",
                3,
                "    ",
                &start_position(),
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
    fn test_render_enforced_indentation_1_line0() {
        for line in ["''''", "'' ''", "'' a''"] {
            assert_eq!(
                render_enforced_indentation(
                    &EnforceIndentation {
                        last_line_break_significant: false,
                        start: "''".to_owned(),
                        end: "''".to_owned(),
                    },
                    line,
                    3,
                    "  ",
                    &start_position(),
                )
                .unwrap(),
                line,
            );
        }
    }

    #[test]
    fn test_render_enforced_indentation_1_line1() {
        for line in ["''''", "'' ''", "'' a''"] {
            assert_eq!(
                render_enforced_indentation(
                    &EnforceIndentation {
                        last_line_break_significant: true,
                        start: "''".to_owned(),
                        end: "''".to_owned(),
                    },
                    line,
                    3,
                    "  ",
                    &start_position(),
                )
                .unwrap(),
                line,
            );
        }
    }

    #[test]
    fn test_render_enforced_indentation_2_lines0() {
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''.
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....a
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''.
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....a
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''"
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''"
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''.
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''"
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''........a
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a
                a
            ''"
        );
    }

    #[test]
    fn test_render_enforced_indentation_2_lines1() {
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''.
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....a
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''.
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....a
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''"
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a''"
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''.
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a''"
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''........a
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a
                a''"
        );
    }

    #[test]
    fn test_render_enforced_indentation_3_lines0() {
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''

''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....

''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....a

''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....
....
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....a
....
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....a
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....
....a
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''........a
....a
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''

....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....

....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....a

....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....
....
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....a
....
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....a
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....
....a
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''........a
....a
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''

....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....

....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''........a

....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a

                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....
....
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''........a
....
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a

                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
........a
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....
........a
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''............a
........a
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
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
    fn test_render_enforced_indentation_3_lines1() {
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''

''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....

''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....a

''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....
....
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....a
....
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....a
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....
....a
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''........a
....a
''"
                .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''

....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....

....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....a

....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....
....
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....a
....
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a

            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....a
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....
....a
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''........a
....a
....''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a
                a
            ''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''

....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

                a''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....

....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

                a''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''........a

....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a

                a''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

                a''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....
....
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''

                a''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''........a
....
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a

                a''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
........a
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a
                a''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''....
........a
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                    a
                a''",
        );
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''............a
........a
....a''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                        a
                    a
                a''",
        );
    }

    #[test]
    fn test_render_enforced_indentation_significant_whitespace0() {
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....................a
.....................
................''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
                 
            ''",
        );
    }

    #[test]
    fn test_render_enforced_indentation_significant_whitespace1() {
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                &"\
''
....................a
.....................
................''"
                    .replace('.', " "),
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                a
                 
            ''",
        );
    }

    #[test]
    fn test_render_enforced_indentation_mixed_whitespace0() {
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: false,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
                     a
                    \t
                ''",
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                 a
                \t
            ''",
        );
    }

    #[test]
    fn test_render_enforced_indentation_mixed_whitespace1() {
        assert_eq!(
            render_enforced_indentation(
                &EnforceIndentation {
                    last_line_break_significant: true,
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                },
                "''
                     a
                    \t
                ''",
                3,
                "    ",
                &start_position(),
            )
            .unwrap(),
            "''
                 a
                \t
            ''",
        );
    }
}
