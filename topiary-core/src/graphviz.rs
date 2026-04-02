//! GraphViz visualisation for our SyntaxTree representation.
//! Named syntax nodes are elliptical; anonymous are rectangular.
use std::{borrow::Cow, fmt, io};

use crate::{FormatterResult, tree_sitter::SyntaxNode};

/// Doubly escapes whitespace (\n and \t) so it is
/// rendered as the escaped value in the GraphViz output
fn escape(input: &str) -> Cow<'_, str> {
    // No allocation happens for an empty string
    let mut buffer = String::new();

    let mut start: usize = 0;
    let length = input.len();

    let append = |buffer: &mut String, from: &mut usize, to: usize, suffix: &str| {
        // Allocate buffer only when necessary
        if buffer.is_empty() {
            // Best case:  length + 1  (i.e., single escaped character in input)
            // Worst case: length * 3  (i.e., every character needs double-escaping)
            // The input is likely to be short, so no harm in reserving for the worst case
            buffer.reserve(length * 3);
        }

        // Decant the unescaped chunk from the input,
        // followed by the escaped suffix provided
        *buffer += &input[*from..to];
        *buffer += suffix;

        // Fast-forward the tracking cursor to the next character
        *from = to + 1;
    };

    for (idx, current) in input.chars().enumerate() {
        match current {
            // Double-escape whitespace characters
            '\n' => append(&mut buffer, &mut start, idx, r#"\\n"#),
            '\t' => append(&mut buffer, &mut start, idx, r#"\\t"#),

            otherwise => {
                // If char::escape_default starts with a backslash, then we
                // have an escaped character and we're off the happy path
                let mut escaped = otherwise.escape_default().peekable();
                if escaped.peek() == Some(&'\\') {
                    append(
                        &mut buffer,
                        &mut start,
                        idx,
                        &otherwise.escape_default().to_string(),
                    );
                }
            }
        }
    }

    if buffer.is_empty() {
        input.into()
    } else {
        // Decant whatever's left of the input into the buffer
        buffer += &input[start..length];
        buffer.into()
    }
}

impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let shape = if self.is_named { "ellipse" } else { "box" };

        writeln!(
            f,
            "  {} [label=\"{}\", shape={shape}];",
            self.id,
            escape(&self.kind)
        )?;

        for child in &self.children {
            writeln!(f, "  {} -- {};", self.id, child.id)?;
            write!(f, "{child}")?;
        }

        Ok(())
    }
}

/// Writes the Graphviz Graph in the dot format to the specified output buffer.
pub fn write(output: &mut dyn io::Write, root: &SyntaxNode) -> std::io::Result<()> {
    writeln!(output, "graph {{")?;
    write!(output, "{root}")?;
    writeln!(output, "}}")?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::escape;
    use std::borrow::Cow;

    #[test]
    fn double_escape() {
        // Property-based testing would be handy, here...
        assert_eq!(escape("foo"), "foo");
        assert_eq!(escape("'"), r#"\'"#);
        assert_eq!(escape("\n"), r#"\\n"#);
        assert_eq!(escape("\t"), r#"\\t"#);
        assert_eq!(
            escape("Here's something\nlonger"),
            r#"Here\'s something\\nlonger"#
        );
    }

    #[test]
    fn escape_borrowed() {
        match escape("foo") {
            Cow::Borrowed("foo") => (),
            _ => panic!("Expected a borrowed, unmodified str"),
        }
    }

    #[test]
    fn escape_owned() {
        match escape("'") {
            Cow::Owned(s) => assert_eq!(s, r#"\'"#),
            _ => panic!("Expected an owned, escaped string"),
        }
    }
}
