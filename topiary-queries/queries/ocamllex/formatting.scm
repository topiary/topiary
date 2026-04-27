; NOTE[regexp] regexp is a unnamed node without a field name, so we typically
; account for places it can be instead of formatting it directly.

(comment) @multi_line_indent_all @allow_blank_line_before @prepend_input_softline @append_input_softline @multi_line_indent_all

(
  (#scope_id! "action")
  (action) @prepend_begin_scope @append_end_scope
)

; If the action spanned multiple lines, add newlines
(action
  (#scope_id! "action")
  "{" @append_spaced_scoped_softline @append_indent_start
  (ocaml) @multi_line_indent_all
  "}" @prepend_spaced_scoped_softline @prepend_indent_end
  ; This last capture name is a bit unfortunate, but it resolves an issue where
  ; the @append_input_softline of the comment is not resolved because the \n is
  ; after it.
) @prepend_input_softline

; Regular expression related rules
(named_regexp
  "let" @append_space
  "=" @prepend_space @append_space
  ; If the regexp spand multiple lines, we may want it to be indented
  ; regexp: (_) @prepend_indent_start @append_indent_end
) @allow_blank_line_before @append_hardline

; Actual regex rules
[
  (character)
  (string)
] @leaf

(string) @prepend_spaced_softline @append_spaced_softline

(character_set
  (#scope_id! "character_set")
  "[" @append_empty_scoped_softline @append_indent_start
  (_) @append_spaced_scoped_softline
  ; Do not append spaces to the last element, then only place a newline
  (_) @append_empty_scoped_softline
  .
  "]" @prepend_indent_end @prepend_antispace
) @prepend_begin_scope @append_end_scope

(aliased_regexp
  "as" @prepend_space @append_space
)

(parenthesized_regexp
  (#scope_id! "parenthesized_regexp")
  "(" @append_empty_scoped_softline @append_indent_start
  ")" @prepend_empty_scoped_softline @prepend_indent_end
) @prepend_begin_scope @append_end_scope @prepend_spaced_softline @append_spaced_softline

(
  (#scope_id! "regexp_alternative")
  (regexp_alternative) @prepend_begin_scope @append_end_scope
)

(regexp_alternative
  (#scope_id! "regexp_alternative")
  "|" @prepend_spaced_scoped_softline @append_space
)

(regexp_concatenation
  (_) @append_spaced_softline
  ; Prevent appending to the last child
  (_)
  .
)

; Lexer related rules
(lexer_definition
  ["rule" "and"] @allow_blank_line_before @append_space @prepend_input_softline
)

(lexer_entry
  (#scope_id! "lexer_entry")
  "=" @prepend_space @append_space
  ["parse" "shortest"] @append_spaced_scoped_softline @append_indent_start
) @prepend_begin_scope @append_end_scope

(lexer_entry
  (lexer_argument) @prepend_space @append_space
)

(lexer_entry
  (#scope_id! "lexer_entry")
  ; TODO: Is this allowed to be a space? Use hardline if not
  "|" @prepend_hardline @allow_blank_line_before
  ; @prepend_space is for NOTE[regexp]
)

(lexer_entry
  ; End indentation after the last child node
  (_) @append_indent_end
  .
)

(lexer_case
  ; The anonymous child of the lexer_case is the regexp
  (_) @prepend_space @append_space
  (action) @prepend_space @prepend_indent_start @append_indent_end
)
