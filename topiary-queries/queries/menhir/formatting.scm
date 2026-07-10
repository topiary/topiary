; -------------------------------------------------------------------------
; Comments
; -------------------------------------------------------------------------
; Comments can span multiple lines. If they do, they should be indented properly
; relative to their surrounding context. We use @multi_line_indent_all to ensure
; block comments align cleanly.
; We allow blank lines before comments for grouping, and use @prepend_input_softline
; to preserve the author's original line breaks.
[
  (comment)
  (ocaml_comment)
] @multi_line_indent_all @allow_blank_line_before @prepend_input_softline

; Appending an input softline ensures comments don't aggressively squash into
; the following syntax elements if they were originally written on separate lines.
(
  [
    (comment)
    (ocaml_comment)
  ] @append_input_softline
)

; Line comments inherently consume the rest of the line, so they must always
; force a hard newline after them to prevent the next node from being absorbed.
(line_comment) @append_hardline

; -------------------------------------------------------------------------
; Declarations (e.g. %token, %type, %start)
; -------------------------------------------------------------------------
; Declarations represent major structural boundaries in the preamble.
; For readability, we enforce a hardline before each declaration and allow
; the author to leave blank lines between them for logical grouping.
(declaration) @prepend_hardline @allow_blank_line_before

; Keywords should be separated from their arguments by a space.
; Example: `%token <type>` instead of `%token<type>`.
(declaration
  [
    "%token"
    "%type"
    "%start"
    "%inline"
    "%left"
    "%right"
    "%nonassoc"
    "%parameter"
  ] @append_space
)

; -------------------------------------------------------------------------
; Headers (%{ ... %})
; -------------------------------------------------------------------------
; The header contains raw OCaml code. We use @multi_line_indent_all so that if
; the injected OCaml spans multiple lines, every line shifts to match the
; surrounding indentation block. The delimiters `%}` and `%{` are placed on
; their own lines to properly frame the code block.
(header
  "%{" @append_hardline
  (ocaml) @multi_line_indent_all
  "%}" @prepend_hardline
) @append_hardline @allow_blank_line_before

; -------------------------------------------------------------------------
; Types (< ... >)
; -------------------------------------------------------------------------
; OCaml types inside `< >` get a trailing space. This ensures separation
; from subsequent tokens (e.g. `%token <int> EOF`).
(type
  "<"
  (ocaml_type)
  ">" @append_space
)

; -------------------------------------------------------------------------
; Tokens & Terminal Aliases
; -------------------------------------------------------------------------
; Token declarations can be written compactly on a single line, or vertically
; across multiple lines. We explicitly define a "declaration" scope to
; isolate line-breaking decisions to this specific token block.
; If the user wrote the tokens on a single line, Topiary keeps them on a single line.
; If the user wrote the tokens across multiple lines, Topiary breaks them all onto
; new lines and applies the indentation (`@append_indent_start`).
; Example single-line: `%token <int> A B C`
; Example multi-line:
; %token <int>
;     A
;     B
(declaration
  (#scope_id! "declaration")
  [
    "%token"
    (type)
  ] @append_indent_start
) @prepend_begin_scope @append_end_scope @append_indent_end

; Linked to the scope above, this directive uses `spaced_scoped_softline`
; so that tokens are separated by spaces if the block is on a single line,
; but explicitly broken onto new lines if the declaration scope is multi-line.
(terminal_alias_attrs
  (#scope_id! "declaration")
) @prepend_spaced_scoped_softline

; If a token has a string alias, we require a space between the token UID
; and the string. Example: `MULT "*"` instead of `MULT"*"`.
(terminal_alias_attrs
  (uid) @append_space
)

(non_terminal) @append_spaced_softline
(strict_actual) @append_spaced_softline

; -------------------------------------------------------------------------
; Rules
; -------------------------------------------------------------------------
; Grammar rules form the core logic of the file. They must begin on a new line
; and can optionally be separated by blank lines for visual clarity.
(old_rule) @append_hardline @allow_blank_line_before

; Optional rule flags (like `public`, `inline`) should be separated from
; the rule name by a space.
(flags) @append_space

; The colon separating a rule name from its productions is padded with an
; input_softline. This allows the author to choose whether to put the first
; production on the same line or on a new line.
(old_rule
  ":" @append_input_softline
)

; -------------------------------------------------------------------------
; Productions (Alternative Branches)
; -------------------------------------------------------------------------
; Alternative productions are separated by `|`.
; We use `input_softline` before the pipe. This respects the author's stylistic
; choice: simple rules can remain compact on one line (e.g., `A: B | C`),
; while complex rules can be formatted vertically.
(old_rule
  "|" @prepend_input_softline @append_space
)

(production_group
  "|" @prepend_input_softline @append_space
)

; -------------------------------------------------------------------------
; Producers (Symbols in a Production)
; -------------------------------------------------------------------------
; Named producers require spaces around the equals sign for readability.
; Example: `e = expr` instead of `e=expr`.
(producer
  (lid) @append_space
  "=" @append_space
)

; Producers must be separated by spaces so they don't merge into one another.
(producer) @append_space

; Menhir allows trailing semicolons in productions. If a semicolon follows
; a producer, the preceding `@append_space` rule would normally push it away
; (e.g. `e = expr ;`). The `@prepend_antispace` acts as a localized vacuum,
; destroying the generated space to snap the semicolon tight: `e = expr;`.
(producer ";" @prepend_antispace)

; -------------------------------------------------------------------------
; Actuals (Rule Arguments)
; -------------------------------------------------------------------------
; When a rule is parameterized, arguments are separated by commas.
; A trailing space ensures they don't cluster (e.g., `A(x, y)`).
(actual
  "," @append_space
)

; -------------------------------------------------------------------------
; Semantic Actions ({ ... })
; -------------------------------------------------------------------------
; Semantic actions embed OCaml code. Just like headers, we apply
; `@multi_line_indent_all` so that if the OCaml code spans multiple lines,
; it shifts appropriately to follow the grammar's indentation depth.
(action
  "{" @append_space
  (ocaml) @multi_line_indent_all
  "}" @prepend_space
)
