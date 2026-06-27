; NOTE There is (currently) no support for line continuations. As such,
; any which are encountered by Topiary will be forcibly collapsed on to
; a single line. (See Issue #172)

; Don't modify strings, heredocs, comments, atomic "words", variable
; expansions or array indices.
[
  (comment)
  (expansion)
  (heredoc_redirect)
  (string)
  (word)
] @leaf

(simple_expansion
  "$"
  .
  (_) @leaf
)

(subscript
  index: (_) @leaf
)

;; Spacing

; Allow blank line before
[
  (c_style_for_statement)
  (case_item)
  (case_statement)
  (command)
  (comment)
  (compound_statement)
  (declaration_command)
  (for_statement)
  (function_definition)
  (if_statement)
  (list)
  (pipeline)
  (redirected_statement)
  (subshell)
  (variable_assignment)
  (while_statement)
] @allow_blank_line_before

; Insert a new line before multi-line syntactic blocks, regardless of
; context
[
  (c_style_for_statement)
  (case_statement)
  (for_statement)
  (function_definition)
  (if_statement)
  (while_statement)
] @prepend_hardline

; Subshells and compound statements should have a new line inserted
; before them when they are top-level constructs. Beyond that level, the
; extra spacing makes the code overly sparse. (This is also a pragmatic
; choice: as we have to avoid the exception of function definitions, the
; list of complementary contexts we'd have to enumerate queries over is
; rather large!)
(program
  [
    (compound_statement)
    (subshell)
  ] @prepend_hardline
)

; A run of "units of execution" (see Commands section, below; sans
; variables which are special) should be interposed by a new line, after
; a multi-line syntactic block or variable.
(
  [
    (c_style_for_statement)
    (case_statement)
    (declaration_command)
    (for_statement)
    (function_definition)
    (if_statement)
    (variable_assignment)
    (while_statement)
  ]
  .
  ; Commands (sans variables)
  [(command) (list) (pipeline) (subshell) (compound_statement) (redirected_statement)] @prepend_hardline
)

; A run of variable declarations and assignments should be interposed by
; a new line, after almost anything else. This makes them stand out.
(
  [
    (c_style_for_statement)
    (case_statement)
    (command)
    (compound_statement)
    (for_statement)
    (function_definition)
    (if_statement)
    (list)
    (pipeline)
    (redirected_statement)
    (subshell)
    (while_statement)
  ]
  .
  [
    (declaration_command)
    (variable_assignment)
  ] @prepend_hardline
)

; Append a space to the following keywords and delimiters
[
  ";"
  "case"
  "declare"
  "do"
  "elif"
  "export"
  "for"
  "if"
  "in"
  "local"
  "readonly"
  "select"
  "then"
  "typeset"
  "until"
  "while"
] @append_space

";" @prepend_antispace

; Prepend a space to intra-statement keywords
[
  "in"
] @prepend_space

;; Comments

; Comments come in two flavours: standalone (i.e., it's the only thing
; on a line, starting at the current indent level); and trailing (i.e.,
; following some other statement on the same line, with a space
; interposed). Bash does not have multi-line comments; they are all
; single-line.
;
; The grammar parses all comments as the (comment) node, which are
; siblings under a common parent.
;
; Formatting Rules:
;
; 1. A comment's contents must not be touched; some (namely the shebang)
;    have a syntactic purpose.
; 2. All comments must end with a new line.
; 3. Comments can be interposed by blank lines, if they exist in the
;    input (i.e., blank lines shouldn't be engineered elsewhere).
; 4. A comment can never change flavour (i.e., standalone to trailing,
;    or vice versa).
; 5. Trailing comments must be interposed by a space (unless they follow
;    a semicolon, but we still insert the space).

; Rule 1: See @leaf rule, above

; Rule 2
(comment) @append_hardline

; Rule 3: See @allow_blank_line_before rule, above.
; FIXME This doesn't quite get us what we want. It's close, but blank
; lines between comments can get consumed.

; Rule 4: We only have to protect against the case of a standalone
; comment, after a statement, being slurped on to that statement's line
; and becoming a trailing comment. That case is satisfied by Rule 5.

; Rule 5
(comment) @prepend_space

;; Compound Statements and Subshells

; Compound statements and subshells are formatted in exactly the same
; way. In a multi-line context, their opening parenthesis triggers a new
; line and the start of an indent block; the closing parenthesis
; finishes that block. In a single-line context, spacing is used instead
; of new lines (NOTE that this is a syntactic requirement of compound
; statements, but not of subshells).
;
; NOTE Despite being isomorphic, the queries for compound statements and
; subshells are _not_ generalised, to ensure parentheses balance.

(compound_statement
  .
  "{" @append_spaced_softline @append_indent_start
  _
  "}" @prepend_spaced_softline @prepend_indent_end
  .
)

(subshell
  .
  "(" @append_spaced_softline @append_indent_start
  _
  ")" @prepend_spaced_softline @prepend_indent_end
  .
)

;; Commands

; "Command" is an epithet for, broadly speaking, a "unit of execution".
; It is such a pervasive and general concept in Bash that we need to
; take care when considering the context. For example, the condition in
; an if statement or while loop is a command, but we don't want to
; insert a new line in these cases.
;
; In terms of the grammar, the following nodes should be considered
; "commands":
;
; * (command)
;   Simple commands (e.g., binaries, builtins, functions, etc.)
;
; * (list)
;   Command lists (i.e., "commands" sequenced by && and ||)
;
; * (pipeline)
;   Command pipelines (i.e., "commands" sequenced by | and |&)
;
; * (subshell)
;   Subshells (i.e., arbitrary code enclosed within parentheses)
;
; * (compound_statement)
;   Compound statements (i.e., arbitrary code enclosed within
;   curly-parentheses)
;
; * (redirected_statement)
;   IO redirection (NOTE These aren't semantically "units of execution"
;   in their own right, but are treated as such due to how the grammar
;   organises them as parent nodes of such units)
;
; * (variable_assignment)
;   Variable assignment (NOTE These aren't "units of execution" at all,
;   but are treated as such to isolate them from their declaration
;   context; see Variables section, below)

; We care about the line spacing of "commands" that appear in any of the
; following contexts:
;
; * Top-level statements
; * Multi-line compound statements and subshells
; * Any branch of a conditional or case statement
; * Loop bodies
; * Multi-line command substitutions
;
; We address each context individually, as there's no way to isolate the
; exceptional contexts, where no line spacing is required. We use custom
; scopes to detect cases where a "command" is followed by a newline in
; the input, because modifications to the grammar since v0.19.0 no
; longer add anonymous '\n' nodes to the tree.

(program
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "program_line_break")
)

(compound_statement
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "compound_statement_line_break")
)

(subshell
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "subshell_line_break")
)

(if_statement
  .
  _
  "then"
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "(if_statement_line_break")
)

(elif_clause
  .
  _
  "then"
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "elif_clause_line_break")
)

(else_clause
  .
  "else"
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "else_clause_line_break")
)

(case_item
  .
  _
  ")"
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "case_item_line_break")
)

(do_group
  .
  "do"
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "do_group_line_break")
)

(command_substitution
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "command_substitution_line_break")
)

; Spaces between named nodes and command list/pipeline delimiters
(list
  [(_) "&&" "||"] @append_space
  .
  _
)

(pipeline
  ["|" "|&"] @prepend_space @append_spaced_softline
)

(pipeline
  .
  (_)
  .
  ["|" "|&"] @append_indent_start
) @append_indent_end

; Prepend the asynchronous operator with a space
; NOTE This can interpose two "commands", but it's rare to see it in the
; wild. As such, we also append a spaced softline
(_
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement)]
  .
  "&" @prepend_space @append_spaced_softline
)

; Spaces between command and its arguments
; NOTE If we treat (command) as a leaf node, then commands are formatted
; as is and the below will be ignored. On balance, I think keeping this
; rule, rather than deferring to the input, is the better choice
; (although it's not without its problems; e.g., see Issue #172).
(command
  (_) @append_space
  .
  (_)
)

; Ensure the negation operator is surrounded by spaces
; NOTE This is a syntactic requirement
(negated_command
  .
  "!" @prepend_space @append_space
)

(command_substitution
  .
  "`" @delete @append_delimiter
  .
  _ @prepend_empty_softline @prepend_indent_start
  (#delimiter! "$(")
)

(command_substitution
  _ @append_empty_softline @append_indent_end
  .
  "`" @delete @append_delimiter
  .
  (#delimiter! ")")
)

; Multi-line command substitutions become an indent block
(command_substitution
  "$(" @append_empty_softline @append_indent_start
  ")" @prepend_empty_softline @prepend_indent_end
)

; Ensure a space interposes command substitutions containing subshells
; This is to remove the ambiguity with $(( ... ))
(command_substitution
  .
  (subshell) @prepend_space
)

(command_substitution
  (subshell) @append_space
  .
)

;; Redirections

; Spaces between command and any redirections (NOTE this will not insert
; a space between the redirection operator and its destination)
(redirected_statement
  (_) @append_space
  .
  (_)
)

; ...with the exceptions of herestrings, that are spaced
(herestring_redirect (_) @prepend_space)

;; Conditionals

; New line after conditionals
[
  (if_statement)
  (elif_clause)
  (else_clause)
] @append_hardline

; New line after "then" and start indent block
[
  (if_statement)
  (elif_clause)
]
"then" @append_hardline @append_indent_start

; New line after "else" and start indent block
(else_clause
  .
  "else" @append_hardline @append_indent_start
)

; Finish indent block at "fi", "else" or "elif"
(if_statement
  [
    "fi"
    (else_clause)
    (elif_clause)
  ] @prepend_indent_end @prepend_hardline
)

; Keep the "if"/"elif" and the "then" on the same line,
; inserting a spaced delimiter when necessary
(_
  ";"* @do_nothing
  .
  "then" @prepend_delimiter
  (#delimiter! "; ")
)

;; Test Commands

(test_command
  "[" @append_space
  "]" @prepend_space
)

(test_command
  "[[" @append_space
  "]]" @prepend_space
)

; NOTE (arithmetic_expansion) seems to take higher priority
; (test_command
;   "((" @append_space
;   "))" @prepend_space
; )

(arithmetic_expansion
  "$[" @delete @append_delimiter
  (#delimiter! "$(( ")
)

(arithmetic_expansion
  "]" @delete @append_delimiter
  (#delimiter! " ))")
)

(arithmetic_expansion
  ["$((" "(("] @append_space
  ["))"] @prepend_space
)

(unary_expression
  "!" @append_space
)

(unary_expression
  (test_operator) @append_space
)

(binary_expression
  left: (_) @append_space
  right: (_) @prepend_space
)

;; Case Statements

; Indentation block between the "in" and the "esac"
(case_statement
  .
  "case"
  .
  _
  .
  "in" @append_hardline @append_indent_start
  _
  "esac" @prepend_hardline @prepend_indent_end
  .
) @append_hardline

; New (soft)line after branch, which starts an indentation block up
; until its end
(case_item
  ")" @append_spaced_softline @append_indent_start
) @append_indent_end

; Ensure case branch terminators appear on their own line, in a
; multi-line context; or, at least, push the next case branch on to a
; new line in a single-line context
; NOTE The terminator is optional in the final condition, which is why
; we deal with closing the indent block above
(case_item
  [
    ";;"
    ";;&"
    ";&"
  ] @append_hardline
  .
)

;; Loops

; Indentation block between the "do" and the "done"
(do_group
  .
  "do" @append_hardline @append_indent_start
  _
  "done" @prepend_hardline @prepend_indent_end
  .
) @append_hardline

; Ensure the word list is delimited by spaces in classic for loops
(for_statement
  value: _* @prepend_space
)

; Surround the loop condition with spaces in C-style for loops
(c_style_for_statement
  initializer: _ @prepend_space
  update: _ @append_space
)

; Keep the loop construct and the "do" on the same line,
; inserting a spaced delimiter when necessary
(_
  ";"* @do_nothing
  .
  (do_group) @prepend_delimiter
  (#delimiter! "; ")
)

;; Function Definitions

; NOTE Much of the formatting work for function definitions is done by
; whatever already-defined queries apply to the function body (e.g.,
; (compound_statement), etc.). All we do here is ensure functions get
; a space between its name and body, a new line afterwards and deleting
; the redundant "function" keyword, if it exists in the input.

; NOTE Technically speaking, a function body can be _any_ compound. For
; example, this is valid Bash:
;
;   my_function() for x in $@; do echo $x; done
;
; However, this form is never seen in the wild and the Tree Sitter Bash
; grammar won't even parse it. It only accepts subshells, compound
; statements and test commands as function bodies.

(function_definition
  body: (_) @prepend_space @append_hardline
)

(function_definition
  .
  (word) @append_delimiter
  .
  (
    "("
    ")"
  )? @do_nothing

  (#delimiter! "()")
)

; NOTE This rule must appear after the above, where the "()" are
; appended, if necessary
(function_definition
  .
  "function" @delete
)

;; Variable Declaration, Assignment and Expansion

; NOTE Assignment only gets a new line when not part of a declaration;
; that is, all the contexts in which units of execution can appear.
; Hence the queries for this are defined above. (My kingdom for a
; negative anchor!)

; Declarations always end with a new line
(
  (declaration_command)
  .
  (_) @prepend_hardline
)

; All declaration arguments must be separated by whitespace
(declaration_command
  (word) @append_space
)

; Multiple variables can be declared (and assigned) at once
(declaration_command
  [
    (variable_name)
    (variable_assignment)
  ] @prepend_space
)

(declaration_command
  (variable_name)? @do_nothing
  .
  (concatenation) @prepend_space
)

; Environment variables assigned to commands inline need to be spaced
(command
  (variable_assignment) @append_space
)

; Multi-line arrays start an indentation block
(array
  "(" @append_empty_softline @append_indent_start
  ")" @prepend_empty_softline @prepend_indent_end
)

; Array elements need to be spaced
(array
  (_) @append_spaced_softline
  .
  (_)
)

; Convert (simple_expansion) into (expansion)s
; NOTE Only convert $word, not special forms like $0, $@, etc.
(simple_expansion
  (#delimiter! "{")
  "$"
  .
  (variable_name) @prepend_delimiter
  (#not-match? @prepend_delimiter "[0-9]")
)

(simple_expansion
  (#delimiter! "}")
  "$"
  .
  (variable_name) @append_delimiter
  (#not-match? @append_delimiter "[0-9]")
)
