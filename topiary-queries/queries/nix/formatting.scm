(_ (string_fragment) @leaf @multi_line_indent_all)

(binding_set) @prepend_indent_start @append_indent_end
(binding_set (binding) @prepend_hardline @append_input_softline)
(binding
  attrpath: (attrpath) @append_space
  "=" @append_space
)
