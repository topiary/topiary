(fenced_code_block
  (info_string
    (language) @injection.language
  )
  (code_fence_content) @injection.content @injection.preserve_indent
  ; mdBook inline macro
  (#not-match? @injection.content "\\{\\{\\#include")
)
