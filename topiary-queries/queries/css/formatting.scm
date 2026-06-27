;; Sometimes we want to indicate that certain parts of our source text should
;; not be formatted, but taken as is. We use the leaf capture name to inform the
;; tool of this.
[
  (integer_value)
  (plain_value)
  (string_value)
  (color_value)
  (float_value)
  (grid_value)
] @leaf

; Append space after colons
(declaration ":" @append_space)
(feature_query
  ":" @append_space
)

; Append space after import
(import_statement "@import" @append_space)
(import_statement ";" @append_hardline)
(import_statement) @prepend_hardline @allow_blank_line_before

; Append space after charset
(charset_statement "@charset" @append_space)
(charset_statement ";" @append_hardline)
(charset_statement) @prepend_hardline @allow_blank_line_before

; Append space after media and hardlines around it
(media_statement "@media" @append_space)
(media_statement "," @append_space)
(media_statement) @allow_blank_line_before @prepend_hardline

; Append space after keyframe and hardlines around it
(keyframes_statement "@keyframes" @append_space)
(keyframes_statement) @prepend_hardline @allow_blank_line_before

; Space around and in binary queries
(binary_query
  "and" @prepend_space @append_space
)
; Space around not in unary queries
(unary_query
  "not" @append_space
)

; Add space before any !important declarations
(important) @prepend_space

; Spacing before and after a rule_set
(rule_set) @allow_blank_line_before @prepend_hardline

; Allow comments to have a blank line before them
(comment) @allow_blank_line_before

(
  (_) @prepend_begin_scope
  .
  (comment) @append_end_scope

  (#scope_id! "left_juxtaposed_comment")
)

(
  (comment) @prepend_spaced_scoped_softline
  (#scope_id! "left_juxtaposed_comment")
)

(
  (comment) @prepend_begin_scope
  .
  (_) @append_end_scope

  (#scope_id! "right_juxtaposed_comment")
)

(
  (comment) @append_spaced_scoped_softline
  (#scope_id! "right_juxtaposed_comment")
)

; Allow blank lines before any declaration in a block except the first one
(block . (declaration) (declaration) @allow_blank_line_before)

; Space before curly and after selectors
(selectors) @append_space
(descendant_selector
  (_) @append_space
  .
  (_)
)
(sibling_selector
  (_) @append_space
  "~" @append_space
  (_)
)
(adjacent_sibling_selector
  (_) @append_space
  "+" @append_space
  (_)
)
(child_selector
  (_) @append_space
  ">" @append_space
  (_)
)

; Indent the declarations in the block
(block
  .
  "{" @append_hardline @append_indent_start @prepend_space
  (_)
  "}" @prepend_hardline @prepend_indent_end @append_hardline
  .
)

; Indent the declarations in the keyframe_block_list
(keyframe_block_list
  .
  "{" @append_hardline @append_indent_start @prepend_space
  (_)
  "}" @prepend_hardline @prepend_indent_end @append_hardline
  .
)

; Always have semicolon after declarations
(
  (declaration) @append_delimiter
  (#delimiter! ";")
  (#not-match? @append_delimiter ";$")
)

; Appends hardline between declaration
(
  (declaration
    ";" @append_hardline
  )
  .
  (comment)? @do_nothing
)

; Add space between values after a property name
(declaration
  (property_name)
  ":" @append_space
  (_) @append_space
)

; Do not add a space between the last value and the ending semicolon
(declaration
  ";" @prepend_antispace
)

(declaration
  "," @prepend_antispace @append_space
)

(arguments
  "," @append_space
)

; Newline between selectors
(selectors
  "," @append_hardline
)

; Space between operators in binary expressions
(binary_expression ["+" "*" "-" "/"] @append_space @prepend_space)

; Space between values in arguments lists that don't always use commas
(call_expression
  (arguments
    (_) @append_space
    .
    ["," ")"]* @do_nothing
  )
)
