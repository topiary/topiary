(fenced_code_block
  (info_string
    (language) @injection.language
  )
  (code_fence_content) @injection.content @keep_whitespace @multi_line_indent_all
  ; mdBook inline macro
  (#not-match? @injection.content "\\{\\{\\#include")
)
