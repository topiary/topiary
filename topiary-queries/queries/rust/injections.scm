; https://docs.rs/serde_json/latest/serde_json/macro.json.html
; https://github.com/helix-editor/helix/blob/43bf7c2dc219606c64003aef21151f49f48d0939/runtime/queries/rust/injections.scm#L42-L52
(
  (macro_invocation
    macro: [
      (scoped_identifier name: (_) @_macro_name)
      (identifier) @_macro_name
    ]
    (token_tree
      (token_tree . ["[" "}"]) @injection.content
    )
  )
  (#eq? @_macro_name "json")
  (#injection_language! "json")
)

; TODO: implement `strip!` or `@injection.combined`
; https://docs.rs/toml/latest/toml/macro.toml.html
; toml::toml! { @injection.content }
; (macro_invocation
;   macro: (scoped_identifier
;     path: (identifier) @crate
;     (#eq? @crate "toml")
;     name: (identifier) @macro
;     (#eq? @macro "toml")
;   )
;   .
;   (token_tree) @injection.content
;   .
;   (#injection_language! "toml")
; )

; TODO: handle injections inside leaves
; https://docs.rs/wasmtime/latest/wasmtime/component/macro.bindgen.html
; https://docs.rs/wit-bindgen/latest/wit_bindgen/macro.generate.html
; serde_json::json!( @injection.content )
; (macro_invocation
;   macro: [
;        (scoped_identifier name: (_) @_macro_name)
;        (identifier) @_macro_name
;      ]
;     (#match? @_macro_name "(bindgen|generate)")
;   (token_tree
;     (token_tree
;       (identifier) @key
;       (#eq? @key "inline")
;       .
;       [
;       ; (string_literal (string_content) @injection.content)
;       (raw_string_literal (string_content) @injection.content)
;       ]
;     )
;   )
;   (#injection_language! "wit")
; )
