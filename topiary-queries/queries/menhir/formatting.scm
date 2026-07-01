(comment) @multi_line_indent_all @allow_blank_line_before @prepend_input_softline @append_input_softline

(line_comment) @append_hardline

(declaration) @append_hardline @allow_blank_line_before

(declaration
  [
    "%token"
    "%type"
    "%start"
    "%inline"
    "%left"
    "%right"
    "%nonassoc"
  ] @append_space
)

(header
  "%{" @append_hardline
  (ocaml) @multi_line_indent_all
  "%}" @prepend_hardline
) @append_hardline @allow_blank_line_before

(type
  "<"
  (ocaml_type)
  ">" @append_space
)

(terminal_alias_attrs) @append_space
(non_terminal) @append_space
(strict_actual) @append_space

(old_rule) @append_hardline @allow_blank_line_before

(flags) @append_space

(old_rule
  ":" @append_hardline
)

(old_rule
  "|" @prepend_hardline @append_space
)

(production_group
  "|" @prepend_hardline @append_space
)

(producer
  (lid) @append_space
  "=" @append_space
)

(producer) @append_space
(producer ";" @prepend_antispace)

(actual
  "," @append_space
)

(action
  "{" @append_space
  (ocaml) @multi_line_indent_all
  "}"
)
