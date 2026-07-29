//! After being split into Atoms, and the queries having been applied this
//! module is responsible for rendering the slice of Atoms back into a displayable
//! format.

use std::{cmp::Ordering, fmt::Write};

use rootcause::prelude::ResultExt;

use crate::{
    Atom, Capitalisation, FormatterError, FormatterResult, MultiLineIndent, multi_line_indent,
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
                multi_line_indent,
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

                let mut content = match multi_line_indent {
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
                    MultiLineIndent::EnforceIndentation(enforce_indentation) => {
                        render_multi_line_string(
                            enforce_indentation,
                            content,
                            indent_level,
                            indent,
                            original_position,
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

/// formats multi line strings.
///
/// `enforce_indentation` contains configuration.
/// `content_original` is the multi line string excluding the delimiters.
/// `indent_level` is the current indentation level of the line containing the start delimiter.
/// `indent` is the white space prefix of a line at indentation level 1.
/// the returned `String` differs only in white space from `content_original`
/// and the returned `String` does include delimiters again.
fn render_multi_line_string(
    enforce_indentation: &multi_line_indent::EnforceIndentation,
    content_original: &str,
    indent_level: usize,
    indent: &str,
    start_position: &Position,
) -> String {
    let multi_line_indent::EnforceIndentation {
        start,
        end,
        last_line_break_significant,
        carriage_return_significant,
        tab_significant,
    } = enforce_indentation;

    let is_insignificant_whitespace = if *tab_significant {
        |c| c == ' '
    } else {
        |c| c == ' ' || c == '\t'
    };

    // split lines, and normalize line endings
    let content: Vec<&str> = if *carriage_return_significant {
        content_original.split("\n").collect() // because we need `DoubleEndedIterator::next_back`.
    } else {
        content_original
            .split("\n")
            .map(|s| s.strip_suffix("\r").unwrap_or(s))
            .collect()
    };

    // early exit with the original content for degenerate multi line string
    if content.len() == 1 {
        return format!("{start}{content_original}{end}");
    }

    let mut content = content.iter().copied();
    let mut buffer = String::new();
    write!(buffer, "{start}").expect("`fmt::Write`ing to a `String` should never fail.");

    // early exit on carriage returns
    if *carriage_return_significant
        && content
            .clone()
            .next()
            .expect("`split` should not produce empty iterators.")
            .ends_with('\r')
    {
        // we cannot introduce `\r\n` line breaks because `\r`s are significant.
        // we do not want to introduce `\n` line breaks because we do not want to introduce inconsistent line endings.
        // so we have to short circuit the rest of the algorithm because it might add line breaks.
        // we might implement some reduced formatting that does not add line breaks in the future if that is considered worth it.
        log::warn!(
            "not formatting the multi line string starting with {:?} at {} \
                because carriage returns become part of the string's value according to the query \
                and the line of the string's start delimiter ends with a carriage return.",
            content_original.chars().take(16).collect::<String>(),
            start_position,
        );
        return format!("{start}{content_original}{end}");
    }

    // skip potential empty first line
    if content
        .clone()
        .next()
        .expect("`split` should not produce empty iterators.")
        .chars()
        .all(is_insignificant_whitespace)
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
        .all(is_insignificant_whitespace);
    if last_line_is_whitespace {
        content.next_back().expect(
            "`split` should not produce empty iterators and `content_collected.len() != 1`.",
        );
    }

    // early exit with empty string for delimiters on successive lines and whitespace characters only
    if content.clone().next().is_none() {
        return format!("{start}{end}");
    }

    // length of the longest prefix-comparable whitespace prefix both in characters and utf 8 bytes
    let mut prefixcomparable_whitespace_prefix_len: usize = 0;
    let mut prefixcomparable_whitespace_prefix_len_utf8 = 0;
    let mut content_collected: Vec<_> = content.clone().map(str::chars).collect();
    loop {
        let mut nexts = content_collected.iter_mut().filter_map(Iterator::next);
        if let Some(first) = nexts.next()
            && is_insignificant_whitespace(first)
            && nexts.all(|c| c == first)
        {
            prefixcomparable_whitespace_prefix_len += 1;
            prefixcomparable_whitespace_prefix_len_utf8 += first.len_utf8();
        } else {
            break;
        }
    }

    // very simple non exhaustive check for mixing of spaces and tabs
    if log::log_enabled!(log::Level::Info) {
        match content
            .clone()
            .filter(|s| !s.chars().all(is_insignificant_whitespace))
            .map(str::chars)
            .map(|s| s.take_while(|c| is_insignificant_whitespace(*c)))
            .map(Iterator::count)
            .min()
            .map(|min_whitespace_prefix_len| {
                prefixcomparable_whitespace_prefix_len.cmp(&min_whitespace_prefix_len)
            }) {
            None => (), // no lines other than whitespace lines
            // do not change the log level without changing it in the if condition 6 lines above.
            Some(Ordering::Less) => log::info!(
                "the multi line string starting with {:?} at {} mixes different whitespace characters like spaces and tabs in its lines' whitespace prefixes. \
                    is this supposed to be indentation? then you should not mix different whitespace characters.",
                content_original.chars().take(16).collect::<String>(),
                start_position,
            ),
            Some(Ordering::Equal) => (),
            Some(Ordering::Greater) => panic!(
                "the common whitespace prefix should be a substring of the shortest whitespace prefix."
            ),
        }
    }

    // strip insignificant prefixes
    let content =
        content.map(|line| &line[line.len().min(prefixcomparable_whitespace_prefix_len_utf8)..]);

    // render output
    for line in content {
        if line.is_empty() {
            writeln!(buffer).expect("`fmt::Write`ing to a `String` should never fail.");
        } else {
            write!(buffer, "\n{}{line}", indent.repeat(indent_level + 1))
                .expect("`fmt::Write`ing to a `String` should never fail.");
        }
    }

    // potentially introduce line break before end delimiter
    #[allow(clippy::nonminimal_bool)]
    if last_line_is_whitespace || (!last_line_is_whitespace && !last_line_break_significant) {
        write!(buffer, "\n{}", indent.repeat(indent_level))
            .expect("`fmt::Write`ing to a `String` should never fail.");
    }

    write!(buffer, "{end}").expect("`fmt::Write`ing to a `String` should never fail.");

    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start_position() -> Position {
        Position { row: 1, column: 1 }
    }

    #[test]
    fn test_render_multi_line_string0() {
        assert_eq!(
            render_multi_line_string(
                &multi_line_indent::EnforceIndentation {
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                    last_line_break_significant: false,
                    carriage_return_significant: false,
                    tab_significant: false,
                },
                "\
''\t
    a
   b
     c
''"
                .strip_prefix("''")
                .unwrap()
                .strip_suffix("''")
                .unwrap(),
                3,
                "    ",
                &start_position(),
            ),
            "''
                 a
                b
                  c
            ''",
        );
    }

    #[test]
    fn test_render_multi_line_string1() {
        assert_eq!(
            render_multi_line_string(
                &multi_line_indent::EnforceIndentation {
                    start: "''".to_owned(),
                    end: "''".to_owned(),
                    last_line_break_significant: false,
                    carriage_return_significant: false,
                    tab_significant: false,
                },
                "\
''x
    a
   b
     c
''"
                .strip_prefix("''")
                .unwrap()
                .strip_suffix("''")
                .unwrap(),
                3,
                "    ",
                &start_position(),
            ),
            "''
                x
                    a
                   b
                     c
            ''"
        );
    }

    #[test]
    fn test_render_multi_line_string_preserve_whitespace() {
        for ((last_line_break_significant, carriage_return_significant), tab_significant) in
            cartesian_product(
                cartesian_product([false, true], [false, true]),
                [false, true],
            )
        {
            assert_eq!(
                render_multi_line_string(
                    &multi_line_indent::EnforceIndentation {
                        start: "''".to_owned(),
                        end: "''".to_owned(),
                        last_line_break_significant,
                        carriage_return_significant,
                        tab_significant,
                    },
                    &"''
........................a
.........................
....................''"
                        .strip_prefix("''")
                        .unwrap()
                        .strip_suffix("''")
                        .unwrap()
                        .replace('.', " "),
                    4,
                    "    ",
                    &start_position(),
                ),
                "''
....................a
.....................
                ''"
                .replace('.', " "),
            );
        }
    }

    #[test]
    fn test_render_multi_line_string_mixed_whitespace() {
        for ((last_line_break_significant, carriage_return_significant), tab_significant) in
            cartesian_product(
                cartesian_product([false, true], [false, true]),
                [false, true],
            )
        {
            assert_eq!(
                render_multi_line_string(
                    &multi_line_indent::EnforceIndentation {
                        start: "''".to_owned(),
                        end: "''".to_owned(),
                        last_line_break_significant,
                        carriage_return_significant,
                        tab_significant,
                    },
                    &"''
.........................a
........................\t
                    ''"
                    .strip_prefix("''")
                    .unwrap()
                    .strip_suffix("''")
                    .unwrap()
                    .replace('.', " "),
                    4,
                    "    ",
                    &start_position(),
                ),
                "''
.....................a
....................\t
                ''"
                .replace('.', " "),
            );
        }
    }

    #[test]
    fn test_render_multi_line_string_significant_whitespace() {
        for ((last_line_break_significant, carriage_return_significant), tab_significant) in
            cartesian_product(
                cartesian_product([false, true], [false, true]),
                [false, true],
            )
        {
            assert_eq!(
                render_multi_line_string(
                    &multi_line_indent::EnforceIndentation {
                        start: "''".to_owned(),
                        end: "''".to_owned(),
                        last_line_break_significant,
                        carriage_return_significant,
                        tab_significant,
                    },
                    &"\
''
\ra
\ra
''"
                    .strip_prefix("''")
                    .unwrap()
                    .strip_suffix("''")
                    .unwrap(),
                    4,
                    "    ",
                    &start_position(),
                ),
                "''
                    \ra
                    \ra
                ''",
            );
        }
    }

    #[test]
    fn test_render_multi_line_string_carriage_return() {
        for (input, output) in [
            (
                "''
                        a\r
                ''",
                "''
                    a\r
                ''",
            ),
            (
                "''\r
                        a
                ''",
                "''\r
                        a
                ''",
            ),
        ] {
            for (last_line_break_significant, tab_significant) in
                cartesian_product([false, true], [false, true])
            {
                assert_eq!(
                    render_multi_line_string(
                        &multi_line_indent::EnforceIndentation {
                            start: "''".to_owned(),
                            end: "''".to_owned(),
                            last_line_break_significant,
                            carriage_return_significant: true,
                            tab_significant,
                        },
                        input
                            .strip_prefix("''")
                            .unwrap()
                            .strip_suffix("''")
                            .unwrap(),
                        4,
                        "    ",
                        &start_position(),
                    ),
                    output,
                );
            }
        }
    }

    #[test]
    fn test_render_multi_line_string_tab() {
        for (last_line_break_significant, carriage_return_significant) in
            cartesian_product([false, true], [false, true])
        {
            assert_eq!(
                render_multi_line_string(
                    &multi_line_indent::EnforceIndentation {
                        start: "''".to_owned(),
                        end: "''".to_owned(),
                        last_line_break_significant,
                        carriage_return_significant,
                        tab_significant: false,
                    },
                    "\
''
\ta
\ta
''"
                    .strip_prefix("''")
                    .unwrap()
                    .strip_suffix("''")
                    .unwrap(),
                    4,
                    "    ",
                    &start_position(),
                ),
                "''
                    a
                    a
                ''",
            );
            assert_eq!(
                render_multi_line_string(
                    &multi_line_indent::EnforceIndentation {
                        start: "''".to_owned(),
                        end: "''".to_owned(),
                        last_line_break_significant,
                        carriage_return_significant,
                        tab_significant: true,
                    },
                    "\
''
\ta
\ta
''"
                    .strip_prefix("''")
                    .unwrap()
                    .strip_suffix("''")
                    .unwrap(),
                    4,
                    "    ",
                    &start_position(),
                ),
                "''
                    \ta
                    \ta
                ''",
            );
        }
    }

    #[test]
    fn test_render_multi_line_string_1_line() {
        for (((last_line_break_significant, carriage_return_significant), tab_significant), line) in
            cartesian_product(
                cartesian_product(
                    cartesian_product([false, true], [false, true]),
                    [false, true],
                ),
                ["''''", "'' ''", "'' a''"],
            )
        {
            assert_eq!(
                render_multi_line_string(
                    &multi_line_indent::EnforceIndentation {
                        start: "''".to_owned(),
                        end: "''".to_owned(),
                        last_line_break_significant,
                        carriage_return_significant,
                        tab_significant,
                    },
                    line.strip_prefix("''").unwrap().strip_suffix("''").unwrap(),
                    3,
                    "  ",
                    &start_position(),
                ),
                line,
            );
        }
    }

    #[test]
    fn test_render_multi_line_string_2_lines0() {
        for (input, output) in [
            (
                "\
''
''", "''''",
            ),
            (
                "\
''.
''", "''''",
            ),
            (
                "\
''....a
''",
                "''
                    a
                ''",
            ),
            (
                "\
''
....''",
                "''''",
            ),
            (
                "\
''.
....''",
                "''''",
            ),
            (
                "\
''....a
....''",
                "''
                    a
                ''",
            ),
            (
                "\
''
....a''",
                "''
                    a
                ''",
            ),
            (
                "\
''.
....a''",
                "''
                    a
                ''",
            ),
            (
                "\
''........a
....a''",
                "''
                        a
                    a
                ''",
            ),
        ] {
            for (carriage_return_significant, tab_significant) in
                cartesian_product([false, true], [false, true])
            {
                assert_eq!(
                    render_multi_line_string(
                        &multi_line_indent::EnforceIndentation {
                            start: "''".to_owned(),
                            end: "''".to_owned(),
                            last_line_break_significant: false,
                            carriage_return_significant,
                            tab_significant,
                        },
                        &input
                            .strip_prefix("''")
                            .unwrap()
                            .strip_suffix("''")
                            .unwrap()
                            .replace('.', " "),
                        4,
                        "    ",
                        &start_position(),
                    ),
                    output,
                );
            }
        }
    }

    #[test]
    fn test_render_multi_line_string_2_lines1() {
        for (input, output) in [
            (
                "\
''
''", "''''",
            ),
            (
                "\
''.
''", "''''",
            ),
            (
                "\
''....a
''",
                "''
                    a
                ''",
            ),
            (
                "\
''
....''",
                "''''",
            ),
            (
                "\
''.
....''",
                "''''",
            ),
            (
                "\
''....a
....''",
                "''
                    a
                ''",
            ),
            (
                "\
''
....a''",
                "''
                    a''",
            ),
            (
                "\
''.
....a''",
                "''
                    a''",
            ),
            (
                "\
''........a
....a''",
                "''
                        a
                    a''",
            ),
        ] {
            for (carriage_return_significant, tab_significant) in
                cartesian_product([false, true], [false, true])
            {
                assert_eq!(
                    render_multi_line_string(
                        &multi_line_indent::EnforceIndentation {
                            start: "''".to_owned(),
                            end: "''".to_owned(),
                            last_line_break_significant: true,
                            carriage_return_significant,
                            tab_significant,
                        },
                        &input
                            .strip_prefix("''")
                            .unwrap()
                            .strip_suffix("''")
                            .unwrap()
                            .replace('.', " "),
                        4,
                        "    ",
                        &start_position(),
                    ),
                    output,
                );
            }
        }
    }

    #[test]
    fn test_render_multi_line_string_3_lines0() {
        for (input, output) in [
            (
                "\
''

''",
                "''

                ''",
            ),
            (
                "\
''....

''",
                "''

                ''",
            ),
            (
                "\
''....a

''",
                "''
                    a

                ''",
            ),
            (
                "\
''
....
''",
                "''

                ''",
            ),
            (
                "\
''....
....
''",
                "''

                ''",
            ),
            (
                "\
''....a
....
''",
                "''
                    a

                ''",
            ),
            (
                "\
''
....a
''",
                "''
                    a
                ''",
            ),
            (
                "\
''....
....a
''",
                "''
                    a
                ''",
            ),
            (
                "\
''........a
....a
''",
                "''
                        a
                    a
                ''",
            ),
            (
                "\
''

....''",
                "''

                ''",
            ),
            (
                "\
''....

....''",
                "''

                ''",
            ),
            (
                "\
''....a

....''",
                "''
                    a

                ''",
            ),
            (
                "\
''
....
....''",
                "''

                ''",
            ),
            (
                "\
''....
....
....''",
                "''

                ''",
            ),
            (
                "\
''....a
....
....''",
                "''
                    a

                ''",
            ),
            (
                "\
''
....a
....''",
                "''
                    a
                ''",
            ),
            (
                "\
''....
....a
....''",
                "''
                    a
                ''",
            ),
            (
                "\
''........a
....a
....''",
                "''
                        a
                    a
                ''",
            ),
            (
                "\
''

....a''",
                "''

                    a
                ''",
            ),
            (
                "\
''....

....a''",
                "''

                    a
                ''",
            ),
            (
                "\
''........a

....a''",
                "''
                        a

                    a
                ''",
            ),
            (
                "\
''
....
....a''",
                "''

                    a
                ''",
            ),
            (
                "\
''....
....
....a''",
                "''

                    a
                ''",
            ),
            (
                "\
''........a
....
....a''",
                "''
                        a

                    a
                ''",
            ),
            (
                "\
''
........a
....a''",
                "''
                        a
                    a
                ''",
            ),
            (
                "\
''....
........a
....a''",
                "''
                        a
                    a
                ''",
            ),
            (
                "\
''............a
........a
....a''",
                "''
                            a
                        a
                    a
                ''",
            ),
        ] {
            for (carriage_return_significant, tab_significant) in
                cartesian_product([false, true], [false, true])
            {
                assert_eq!(
                    render_multi_line_string(
                        &multi_line_indent::EnforceIndentation {
                            start: "''".to_owned(),
                            end: "''".to_owned(),
                            last_line_break_significant: false,
                            carriage_return_significant,
                            tab_significant,
                        },
                        &input
                            .strip_prefix("''")
                            .unwrap()
                            .strip_suffix("''")
                            .unwrap()
                            .replace('.', " "),
                        4,
                        "    ",
                        &start_position(),
                    ),
                    output,
                );
            }
        }
    }

    #[test]
    fn test_render_multi_line_string_3_lines1() {
        for (input, output) in [
            (
                "\
''

''",
                "''

                ''",
            ),
            (
                "\
''....

''",
                "''

                ''",
            ),
            (
                "\
''....a

''",
                "''
                    a

                ''",
            ),
            (
                "\
''
....
''",
                "''

                ''",
            ),
            (
                "\
''....
....
''",
                "''

                ''",
            ),
            (
                "\
''....a
....
''",
                "''
                    a

                ''",
            ),
            (
                "\
''
....a
''",
                "''
                    a
                ''",
            ),
            (
                "\
''....
....a
''",
                "''
                    a
                ''",
            ),
            (
                "\
''........a
....a
''",
                "''
                        a
                    a
                ''",
            ),
            (
                "\
''

....''",
                "''

                ''",
            ),
            (
                "\
''....

....''",
                "''

                ''",
            ),
            (
                "\
''....a

....''",
                "''
                    a

                ''",
            ),
            (
                "\
''
....
....''",
                "''

                ''",
            ),
            (
                "\
''....
....
....''",
                "''

                ''",
            ),
            (
                "\
''....a
....
....''",
                "''
                    a

                ''",
            ),
            (
                "\
''
....a
....''",
                "''
                    a
                ''",
            ),
            (
                "\
''....
....a
....''",
                "''
                    a
                ''",
            ),
            (
                "\
''........a
....a
....''",
                "''
                        a
                    a
                ''",
            ),
            (
                "\
''

....a''",
                "''

                    a''",
            ),
            (
                "\
''....

....a''",
                "''

                    a''",
            ),
            (
                "\
''........a

....a''",
                "''
                        a

                    a''",
            ),
            (
                "\
''
....
....a''",
                "''

                    a''",
            ),
            (
                "\
''....
....
....a''",
                "''

                    a''",
            ),
            (
                "\
''........a
....
....a''",
                "''
                        a

                    a''",
            ),
            (
                "\
''
........a
....a''",
                "''
                        a
                    a''",
            ),
            (
                "\
''....
........a
....a''",
                "''
                        a
                    a''",
            ),
            (
                "\
''............a
........a
....a''",
                "''
                            a
                        a
                    a''",
            ),
        ] {
            for (carriage_return_significant, tab_significant) in
                cartesian_product([false, true], [false, true])
            {
                assert_eq!(
                    render_multi_line_string(
                        &multi_line_indent::EnforceIndentation {
                            start: "''".to_owned(),
                            end: "''".to_owned(),
                            last_line_break_significant: true,
                            carriage_return_significant,
                            tab_significant,
                        },
                        &input
                            .strip_prefix("''")
                            .unwrap()
                            .strip_suffix("''")
                            .unwrap()
                            .replace('.', " "),
                        4,
                        "    ",
                        &start_position(),
                    ),
                    output,
                );
            }
        }
    }

    fn cartesian_product<AS, BS>(
        list_a: AS,
        list_b: BS,
    ) -> impl Iterator<
        Item = (
            <AS::IntoIter as Iterator>::Item,
            <BS::IntoIter as Iterator>::Item,
        ),
    >
    where
        AS: IntoIterator,
        BS: IntoIterator,
        BS::IntoIter: Clone,
        <AS::IntoIter as Iterator>::Item: Clone,
    {
        cartesian_product_internal(list_a.into_iter(), list_b.into_iter())
    }

    fn cartesian_product_internal<AS, BS>(
        list_a: AS,
        list_b: BS,
    ) -> impl Iterator<Item = (AS::Item, BS::Item)>
    where
        AS: Iterator,
        BS: Iterator + Clone,
        AS::Item: Clone,
    {
        list_a.flat_map(move |a| list_b.clone().map(move |b| (a.clone(), b)))
    }
}
