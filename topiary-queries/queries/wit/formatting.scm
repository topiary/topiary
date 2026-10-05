; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
[
  (block_comment)
  (line_comment)
] @leaf

(block_comment) @multi_line_indent_all
; Allow line break after block comments
(
  (block_comment)
  .
  _ @prepend_input_softline
)

; Input softlines before and after all comments. This means that the input
; decides if a comment should have line breaks before or after. A line comment
; always ends with a line break.
[
  (block_comment)
  (line_comment)
] @prepend_input_softline

; Allow blank line before
[
  (interface_item)
  (world_item)
  (nested_package_definition)
  (package_decl)
  (typedef_item)
  (gate_item)
  (external_id)
  (resource_method)
  (block_comment)
  (line_comment)
] @allow_blank_line_before

[
  "use"
  "func"
  "static"
  "async"
  "type"
  "interface"
  "world"
  "package"
  "resource"
  "record"
  "enum"
  "flags"
  "variant"
  "include"
  "import"
  "export"
  "as"
  "with"
  "->"
  "="
] @prepend_space @append_space

; Append line breaks. If there is a comment following, we don't add anything,
; because the input softlines and spaces above will already have sorted out the
; formatting.
(
  (block_comment)?
  .
  [
    (package_items)
    (statement)
    (func_item)
    (world_definition)
    (typedef_item)
    (gate_item)
    (external_id)
    (resource_method)
    (line_comment)
    (variant_items)
    (enum_items)
    (use_item)
  ] @append_hardline
  .
  [
    (block_comment)
    (line_comment)
  ]* @do_nothing
)

(line_comment) @append_hardline

; A block comment that trails a comma (e.g. an enum/variant case) belongs with
; the preceding item, so break the line after it.
(
  ","
  .
  (block_comment) @append_hardline
)

; Always leave a space before an opening `{`, even when the body is empty
; (the `{ _ }` pattern below only fires for non-empty bodies).
(body "{" @prepend_space)
(nested_package_definition "{" @prepend_space)

(body
  .
  "{" @append_hardline @append_indent_start
  _
  "}" @prepend_hardline @prepend_indent_end
  .
)

(nested_package_definition
  .
  (decl_head)
  "{" @append_hardline @append_indent_start
  _
  "}" @prepend_hardline @prepend_indent_end
  .
)

(definitions
  .
  "{" @append_empty_softline @append_indent_start
  _
  "}" @prepend_empty_softline @prepend_indent_end
  .
)

(
  [
    (enum_case)
    (record_field)
    (variant_case)
    (flags_field)
  ]
  .
  "," @append_hardline
  .
  [
    (block_comment)
    (line_comment)
  ]* @do_nothing
)
; ==================
; Delimiters
; ==================

(param_list
  "," @append_spaced_softline
  .
  [(block_comment) (line_comment)]* @do_nothing
)

(definitions
  (_)
  "," @append_spaced_softline
  .
  [
    (block_comment)
    (line_comment)
  ]* @do_nothing
)
[
  ">"
  ")"
  ","
  ";"
  (param_list)
] @prepend_antispace

"," @append_space

[
  "<"
  "("
] @append_antispace
(param_list
  .
  "(" @append_empty_softline @append_indent_start
  ")" @prepend_indent_end @prepend_empty_softline
  .
)

; Colon should have whitespace trimmed for URI separator
; pkg & use nodes
(decl_head
  ["@" ":" "/"] @prepend_antispace @append_antispace
)
(use_path
  ["@" ":" "/"] @prepend_antispace @append_antispace
)
(record_field
  ":" @prepend_antispace @append_space
)
(named_type
  ":" @prepend_antispace @append_space
)
; Space after the `:` separating an import/export name from its extern_type
(import_item
  ":" @prepend_antispace @append_space
)
(export_item
  ":" @prepend_antispace @append_space
)
; Function signatures need proper spacing
[
  "@"
  "/"
] @prepend_antispace @append_antispace

; Dot should have no space for use paths

; ==================
; Trailing Commas
; ==================
(definitions (_) "," @delete . "}" (#single_line_only!))
(definitions "," @append_spaced_softline (#single_line_only!))
(param_list
  (#delimiter! ",")
  (named_type) @append_delimiter
  .
  ","? @do_nothing
  .
  [(block_comment) (line_comment)]*
  .
  ")"
  .
  (#multi_line_only!)
)

(
  (#delimiter! ",")
  [
    (variant_case)
    (enum_case)
    (flags_field)
    (record_field)
    (use_names_item)
    (include_names_item)
  ] @append_delimiter
  .
  ","? @do_nothing
  (#multi_line_only!)
)

; Keep inline comments on same line as export/import statements
([(import_item) (export_item)] . (line_comment) @prepend_space)
