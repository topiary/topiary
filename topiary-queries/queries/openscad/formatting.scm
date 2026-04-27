; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
[
  (block_comment)
  (line_comment)
  (string)
] @leaf

; Allow blank line before
[
  (use_statement)
  (intersection_for_block)
  (for_block)
  (if_block)
  (let_block)
  (assign_block)
  (union_block)
  (transform_chain)
  (include_statement)
  (assert_statement)
  (line_comment)
  (block_comment)
  (function_item)
  (module_item)
  (expression)
  (var_declaration)
] @allow_blank_line_before

; Keywords

; Surround spaces
[
  "module"
  "let"
  "include"
  "assign"
  "use"
  "each"
  "else"
  "if"
  "||"
  "&&"
  "=="
  "!="
  "<"
  ">"
  "<="
  ">="
  "+"
  "-"
  "*"
  "/"
  "%"
  "^"
  "="
  "?"
  ":"
  (assignments)
  ((parenthesized_expression) . _)
] @prepend_space @append_space

; Colon should have whitespace trimmed in a range delimiter
(range ":" @prepend_antispace @append_antispace)

; Input softlines before and after all comments. This means that the input
; decides if a comment should have line breaks before or after. A line comment
; always ends with a line break.
[
  (block_comment)
  (line_comment)
] @prepend_input_softline

; Append line breaks. If there is a comment following, we don't add anything,
; because the input softlines and spaces above will already have sorted out the
; formatting.
(
  [
    (transform_chain)
    (var_declaration)
    (function_item)
    (module_item)
    (intersection_for_block)
    (for_block)
    (if_block)
    (let_block)
    (assign_block)
    (union_block)
    (use_statement)
    (include_statement)
    (assert_statement)
  ] @append_spaced_softline
  .
  [
    "else"
    (block_comment)
    (line_comment)
    ";"
  ]* @do_nothing
)

(line_comment) @append_hardline

; Allow line break after block comments
(
  (block_comment)
  .
  _ @prepend_input_softline
)

; Append softlines, unless followed by comments.
; When binding multiple values in a let block, allow new lines between the bindings.
(list
  "[" @append_indent_start @append_empty_softline @append_antispace
  "]" @prepend_indent_end @prepend_empty_softline @prepend_antispace
)
; to avoid having a list that directly follows a let_expression look visually unindented,
; add another level of indentation:
; let (
;   u = true,
; )[
;  [1, 0, 0, 0],
;  [0, 1, 0, 0],
;  [0, 0, 1, 0],
;  [0, 0, 0, 1],
; ];
(let_expression
  (list
    "[" @append_indent_start
    "]" @prepend_indent_end
  )
)

(range
  "[" @append_antispace
  "]" @prepend_antispace
)
(assignments "," @append_spaced_softline . [(block_comment) (line_comment)]* @do_nothing)
(parameters "," @append_spaced_softline . [(block_comment) (line_comment)]* @do_nothing)

; Never put a space before a comma semicolon or period/full stop
[
  ","
  ";"
  "."
] @prepend_antispace

; Don't insert spaces between the operator and their expression operand
; '-x' v.s. '- x'
(unary_expression _ @append_antispace . (expression))

; ================================================================================
; functions & modules
; ================================================================================
(function_item "function" @append_space)

; indent the body of a function
(function_item
  (parameters)
  .
  "=" @append_spaced_softline @append_indent_start
  (expression)
  ";" @prepend_indent_end
)

; ===
; function literals
; ===
(function_lit (parameters) @append_spaced_softline)
(function_call (parenthesized_expression) @append_antispace (arguments))

; ===
; module_item, module_call, and transform_chain
; ===

; everything except `union_block` after a for/if/else statement should be a spaced_softline
(module_item
  body: [
    (for_block)
    (intersection_for_block)
    (if_block)
    (let_block)
    (assign_block)
    (transform_chain)
    (include_statement)
    (assert_statement)
  ] @prepend_indent_start @append_indent_end
)

; module calls in a transformation chain will follow each other
; sometimes staying on the same line and sometimes having a linebreak,
; each linebreak typically also starts an indent scope
(transform_chain) @prepend_input_softline
(transform_chain
  (modifier)*
  (module_call) @append_indent_start
  (transform_chain) @append_indent_end
)

; ================================================================================
; blocks/expressions/statements
; ================================================================================

; Let child nodes handle indentation
(var_declaration . (assignment . (identifier) . "=" @append_input_softline))

(assignments) @append_space
; OS2021 does not support trailing commas in assignments
; https://github.com/Leathong/openscad-LSP/issues/51#issuecomment-2891821939
; (assignments
;   (#delimiter! ",")
;   (assignment) @append_delimiter
;   .
;   ","? @do_nothing
;   .
;   [(block_comment) (line_comment)]*
;   .
;   ")"
;   .
;   (#multi_line_only!)
; )
(assignments
  .
  "(" @append_empty_softline @append_indent_start
  ")" @prepend_indent_end @prepend_empty_softline
  .
)
(assignments "," @delete . ")" . (#single_line_only!))

(arguments "," @append_input_softline)
(arguments "," @delete . ")" . (#single_line_only!))
(arguments
  .
  "(" @append_empty_softline @append_indent_start @append_antispace
  ")" @prepend_indent_end @prepend_empty_softline @prepend_antispace
  .
)
; OS2021 does not support trailing commas in arguments
; (arguments
;   (#delimiter! ",")
;   (expression) @append_delimiter
;   .
;   ","? @do_nothing
;   .
;   [(block_comment) (line_comment)]*
;   .
;   ")"
;   .
;   (#multi_line_only!)
; )

(parameters "," @append_input_softline)
(parameters "," @delete . ")" . (#single_line_only!))
(parameters
  .
  "(" @append_empty_softline @append_indent_start
  ")" @prepend_indent_end @prepend_empty_softline
  .
)

(parenthesized_expression
  .
  "(" @append_empty_softline @append_indent_start
  ")" @prepend_indent_end @prepend_empty_softline
  .
)
(list "," @delete . "]" . (#single_line_only!))

(list
  (#delimiter! ",")
  [(expression) (list_comprehension)] @append_delimiter
  .
  ","? @do_nothing
  .
  [(block_comment) (line_comment)]*
  .
  "]"
  .
  (#multi_line_only!)
)
(list "," @append_spaced_softline . [(block_comment) (line_comment)]* @do_nothing)

; differentiate parameter definitions from parameter invocation,
; module/function definitions have param separation while
; module/function calls have none, this space on chained function/module calls
; and provides visual distinction between definitions and calls
(arguments
  (assignment
    "=" @append_antispace @prepend_antispace
  )
)

(union_block
  .
  "{" @append_spaced_softline @append_indent_start @prepend_space
  _
  "}" @prepend_spaced_softline @prepend_indent_end
  .
)

; everything except `union_block` after a for/if/else statement should be a spaced_softline
(if_block
  (parenthesized_expression) @append_spaced_softline @append_indent_start
  .
  [
    (for_block)
    (intersection_for_block)
    (if_block)
    (let_block)
    (assign_block)
    (transform_chain)
    (include_statement)
    (assert_statement)
  ] @append_indent_end @append_spaced_softline
)
(
  "else" @append_spaced_softline @append_indent_start
  .
  [
    (for_block)
    (intersection_for_block)
    (let_block)
    (assign_block)
    (transform_chain)
    (include_statement)
    (assert_statement)
  ] @append_indent_end
) @prepend_spaced_softline

; scope is triggered by the presence of a (union_block) consequce on the intersection_for_block;
; and extends to the "else if" and "else" portions
(
  "else"? @do_nothing
  (if_block
    (#scope_id! "if_union")
  ) @prepend_begin_scope @append_end_scope
)

(if_block
  (#scope_id! "if_union")
  (_
    .
    "{" @append_spaced_scoped_softline
    "}" @prepend_spaced_scoped_softline
    .
  )
)

(for_block
  (assignments) @append_spaced_softline @append_indent_start
  .
  [
    (for_block)
    (intersection_for_block)
    (if_block)
    (let_block)
    (assign_block)
    (transform_chain)
    (include_statement)
    (assert_statement)
  ] @append_indent_end @append_spaced_softline
)
; modifiers
(modifier) @append_antispace

; echo/assert statements can often be chained, use indentation on assignment
(var_declaration
  (assignment
    "=" @append_spaced_softline
    value: [
      (assert_expression)
      (echo_expression)
      (ternary_expression)
    ] @prepend_indent_start
  )
  ";" @prepend_indent_end
  .
)

(assert_expression expression: (_) @prepend_spaced_softline)
(assert_statement statement: (_) @prepend_spaced_softline)
(echo_expression expression: (_) @prepend_spaced_softline)

; ternary expressions
(ternary_expression
  ":" @prepend_spaced_softline
)
; Prettier style ternaries https://prettier.io/blog/2023/11/13/curious-ternaries
(ternary_expression
  "?" @append_input_softline @append_indent_start
  ":" @prepend_indent_end
)
