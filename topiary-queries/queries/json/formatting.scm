; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
(string) @leaf

; Append space after colons
":" @append_space

; We want every object and array to have the { start a softline and an indented
; block. So we match on the named object/array followed by the first anonymous
; node { or [.

; We do not want to add spaces or newlines in empty objects and arrays,
; so we add the newline and the indentation block only if there is a pair in
; the object (or a value in the array).
(object
  .
  "{" @append_spaced_softline @append_indent_start
  (pair)
  "}" @prepend_spaced_softline @prepend_indent_end
  .
)

(array
  .
  "[" @append_spaced_softline @append_indent_start
  (_value)
  "]" @prepend_spaced_softline @prepend_indent_end
  .
)

; Pairs should always end with a softline.
; Pairs come in two kinds, ones with a trailing comma, and those without.
; Those without are the last pair of an object,
; and the line is already added by the closing curly brace of the object.
(object
  "," @append_spaced_softline
)

; Items in an array must have a softline after. See also the pairs above.
(array
  "," @append_spaced_softline
)
