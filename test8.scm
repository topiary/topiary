(string_fragment) @keep_whitespace
(indented_string_expression
  . (_) @prepend_indent_start)
(indented_string_expression
  (_) @append_indent_end .)
