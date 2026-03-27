; This query file is used to format trees produced by two different grammars:
; - the grammar for OCaml interface files `tree_sitter_ocaml::language_ocaml_interface()`
; - the grammar for OCaml implementation files `tree_sitter_ocaml::language_ocaml()`

; On certain cases, some files may correctly parse with both grammars, but produce
; different syntax trees. We want those files to be formatted the same way,
; independently of the grammar used to parse them.

; To ensure this property, we introduce equivalence classes on syntax node types.
; Simply put, if two node types are in the same equivalence class, then they must
; appear in the same queries.

; If you add or modify a query containing a node type in one of the equivalence classes,
; please make sure to add or modify the corresponding queries for all other node types
; in the same equivalence class.

; The equivalence classes are the following:
; - include_module, include_module_type
; - module_path, extended_module_path, module_type_path
; - module_name, module_type_name

; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
(
  [
    (character)
    (quoted_string)
    (signed_number)
    (string)
  ]
) @leaf

(comment) @multi_line_indent_all

; line number directives must be alone on their line, and can't be indented
(line_number_directive) @single_line_no_indent

; Allow blank line before
[
  (class_definition)
  (class_initializer)
  (class_type_definition)
  (comment)
  (exception_definition)
  (external)
  (floating_attribute)
  ; start equivalence class
  (include_module)
  (include_module_type)
  ; end equivalence class
  (inheritance_definition)
  (inheritance_specification)
  (instance_variable_definition)
  (method_definition)
  (module_definition)
  (module_type_definition)
  (open_module)
  (type_definition)
  (value_definition)
  (value_specification)
] @allow_blank_line_before

; The `;;` top-level operator should be alone on its line,
; possibly separated by blank lines above and below.
(
  ";;" @allow_blank_line_before @prepend_hardline @append_hardline
  .
  _ @allow_blank_line_before
)

; In a definition including several mutually recursive functions,
; one can skip a line before each of them.
(value_definition
  "and" @allow_blank_line_before
)
(value_definition
  (let_and_operator) @allow_blank_line_before
)

; In a definition including several mutually recursive types,
; one can skip a line before each of them.
(type_definition
  "and" @allow_blank_line_before
)

; Append line breaks. If there is a comment following, we don't add anything,
; because the input softlines and spaces above will already have sorted out the
; formatting.
(
  [
    (exception_definition)
    (external)
    (module_definition)
    (module_type_definition)
    (type_definition)
  ] @append_spaced_softline
  .
  "in"? @do_nothing
  .
  (comment)* @do_nothing
)
; Also append line breaks after open_module, except when it's
; preceded by "let", because in this case it's in a let_open_expression.
(
  "let"? @do_nothing
  .
  (open_module) @append_hardline
  .
  (comment)* @do_nothing
)

; Append line break after module include, except if it's alone in a single-lined struct
(
  [
    ; start equivalence class
    (include_module)
    (include_module_type)
    ; end equivalence class
  ] @append_hardline
  .
  "end"? @do_nothing
)
(structure
  "struct"
  .
  [
    ; start equivalence class
    (include_module)
    (include_module_type)
    ; end equivalence class
  ] @append_spaced_softline
  .
  "end"
)

; Consecutive definitions must be separated by line breaks
(
  [
    (value_definition)
    (value_specification)
  ] @append_spaced_softline
  .
  [
    (exception_definition)
    (external)
    (module_definition)
    (module_type_definition)
    (open_module)
    (type_definition)
    (value_definition)
    (value_specification)
  ]
)

; Surround spaces
; A space is put after, except when followed by a PPX "%".
(
  [
    "and"
    "as"
    "assert"
    "class"
    "constraint"
    "downto"
    "effect"
    "else"
    "exception"
    "external"
    (floating_attribute)
    "for"
    "fun"
    "function"
    "if"
    "in"
    "include"
    ; Infix operators
    (pow_operator)
    (mult_operator)
    (add_operator)
    (concat_operator)
    (rel_operator)
    (and_operator)
    (or_operator)
    (assign_operator)
    "inherit"
    "initializer"
    (item_attribute)
    "let"
    "match"
    "method"
    "module"
    (module_parameter)
    "mutable"
    "new"
    "nonrec"
    "object"
    "of"
    "open"
    (parameter)
    "private"
    "rec"
    "sig"
    "then"
    "to"
    "try"
    "type"
    "val"
    "virtual"
    "when"
    "while"
    "with"
    "*"
    "="
    "|"
    "->"
    "<-"
    ":"
    ";"
    "+="
    ":="
    ":>"
    "::"
    "[>"
    "[<"
  ] @append_space
  .
  "%"? @do_nothing
)

; Prepend a space. Note that these nodes are not expected to come after an open parenthesis,
; or if they do, we still want a space before.
[
  "as"
  (attribute)
  "constraint"
  "do"
  "done"
  "downto"
  "else"
  (floating_attribute)
  "in"
  (item_attribute)
  (module_parameter)
  "nonrec"
  "of"
  (pow_operator)
  (mult_operator)
  (add_operator)
  (concat_operator)
  (rel_operator)
  (and_operator)
  (or_operator)
  (assign_operator)
  (parameter)
  "rec"
  "then"
  "to"
  "virtual"
  "when"
  "with"
  "="
  "|"
  "->"
  "<-"
  "+="
  ":="
  ":>"
  "::"
] @prepend_space

; let-like and and-like operators are only followed by a closing parenthesis
; during their definition, in which case no space must be appended.
; space must be appended otherwise
(
  (let_and_operator) @append_space
  .
  ")"* @do_nothing
)
(
  (let_operator) @append_space
  .
  ")"* @do_nothing
)

; Put a space after commas, except the last one.
(
  "," @append_space
  .
  (_)
)

; Put a space after the dot in polymorphic function types
;
; let my_const :
;   type a b. a: a -> b: b -> a =
;   fun ~a ~b -> a
(polymorphic_type
  "." @append_space
)

; Certain elements must be separated by spaces.
(
  [
    (boolean)
    (character)
    (class_path)
    (class_type_path)
    (constructed_type)
    (constructor_path)
    (extension)
    (field_get_expression)
    (hash_type)
    (labeled_argument)
    ; start equivalence class
    (extended_module_path)
    (module_path)
    (module_type_path)
    ; end equivalence class
    (number)
    (parenthesized_expression)
    (parenthesized_pattern)
    (polymorphic_variant_type)
    (quoted_string)
    (string)
    (tag)
    (type_constructor)
    (type_constructor_path)
    (typed_expression)
    (type_variable)
    (value_name)
    (value_path)
    (value_pattern)
    ")"
    "]"
  ] @append_space
  .
  [
    ; Don't add any space just before these.
    ","
    ";"
    "."
    ".."
    ")"
    "]"
  ]* @do_nothing
  .
  [
    (abstract_type)
    (boolean)
    (character)
    (class_name)
    (class_path)
    (class_type_name)
    (class_type_path)
    (constructed_type)
    (constructor_path)
    (constructor_pattern)
    (field_get_expression)
    (local_open_pattern)
    (labeled_argument)
    (list_pattern)
    (record_pattern)
    ; start equivalence class
    (extended_module_path)
    (module_path)
    (module_type_path)
    ; end equivalence class
    (number)
    (parenthesized_expression)
    (parenthesized_pattern)
    (polymorphic_variant_type)
    (prefix_expression)
    (quoted_string)
    (range_pattern)
    (string)
    (tag)
    (type_constructor)
    (type_constructor_path)
    (type_variable)
    (typed_expression)
    (value_name)
    (value_path)
    (value_pattern)
    "("
    "{"
  ]
)

; When one uses a language extension, we always put a space after the extension name,
; if this name ends by the '?' or ':' token, the space comes after it.
;
(
  (attribute_id) @append_space
  .
  (attribute_payload
    [
      "?"
      ":"
    ]
  )* @do_nothing
)
(attribute_payload
  [
    "?"
    ":"
  ] @append_space
)

; Some kinds of colons should have a space in front.
(
  [
    (constructor_name)
    (fun_expression)
    (module_name)
    (parenthesized_expression)
    (value_name)
    (value_pattern)
  ] @append_space
  .
  ":"
)

; Keep spacing between pointy brackets when used in object types.
;
; This is syntactically correct:
;   let obj_id (obj : < .. >) = obj
; This is not:
;   let obj_id (obj : <..>) = obj
(object_type
  "<" @append_space
  ">" @prepend_space
)

; Softlines. These become either a space or a newline, depending on whether we
; format their node as single-line or multi-line. If there is a comment
; following, we don't add anything, because they will have their own line break
; processing applied to them.
;
; If those keywords are followed by a ppx extension,
; the soft linebreak comes after it.
(
  [
    "begin"
    "do"
    "struct"
    "then"
    "with"
    ";"
  ] @append_spaced_softline
  .
  [
    (attribute)
    (comment)
    "%"
  ]* @do_nothing
)

(
  [
    "begin"
    "do"
    "struct"
    "then"
    "with"
    ";"
  ]
  .
  "%"
  .
  (attribute_id) @append_spaced_softline
  .
  (comment)* @do_nothing
)

; only add softlines after "else" if it's not part of an "else if" construction
(
  "else" @append_spaced_softline
  .
  [
    (comment)
    (if_expression)
  ]? @do_nothing
)

; ":" must not always be followed by a softline, we explicitly enumerate
; the contexts where it must happen
(value_specification
  ":" @append_spaced_softline
)
(module_binding
  ":" @append_input_softline
)
(field_declaration
  ":" @append_spaced_softline
  [
    (constructed_type
      ; We only care about constructed types which add indentation
      [
        "("
        (polymorphic_variant_type)
        (record_declaration)
      ]
    )
    (polymorphic_variant_type)
    (record_declaration)
  ]? @do_nothing
)
; We also want to add the softline when the following list-like structure is single-line
(field_declaration
  ":" @append_spaced_softline
  [
    (constructed_type
      ; We only care about constructed types which add indentation
      [
        "("
        (polymorphic_variant_type)
        (record_declaration)
      ]
    )
    (polymorphic_variant_type)
    (record_declaration)
  ]
  (#single_line_scope_only! "dangling_list_like")
)
(typed_label
  ":" @append_spaced_softline
)
(constructor_declaration
  ":" @append_spaced_softline
)

; "->" must not always be followed by a softline, we explicitly enumerate
; the contexts where it must happen
(fun_expression
  "->" @append_spaced_softline
)
(match_case
  "->" @append_spaced_softline
)
(constructor_declaration
  "->" @append_spaced_softline
)

; Always put softlines before these:
[
  "and"
  "done"
  "end"
  (else_clause)
  (item_attribute)
  "*"
  "|"
] @prepend_spaced_softline

[
  (match_expression)
] @prepend_empty_softline

; Softline before the first match case
;
; let type_constant = function
;   Const_int _ -> instance Predef.type_int
;   | Const_char _ -> instance Predef.type_char
;
(match_expression
  "|"* @do_nothing
  .
  (match_case) @prepend_spaced_softline
)

(match_expression
  "|" @prepend_empty_softline
)

; Allow (and enforce) the optional "|" before the first match case
; if and only if the context is multi-line
(
  "with"
  .
  "|" @delete
  .
  (match_case)
  (#single_line_only!)
)

(
  "with"
  .
  "|"? @do_nothing
  .
  (match_case) @prepend_delimiter
  (#delimiter! "| ") ; sic
  (#multi_line_only!)
)

; Same as above, with polymorphic variant type, only in the contexts where
; `dangling_list_like` can be defined.
(
  [
    ":"
    "="
  ]
  .
  (polymorphic_variant_type
    "["
    .
    "|" @delete
    .
    (tag_specification)
    (#single_line_scope_only! "dangling_list_like")
  )
)
(
  [
    ":"
    "="
  ]
  .
  (polymorphic_variant_type
    "["
    .
    "|"? @do_nothing
    .
    (tag_specification) @prepend_delimiter
    (#delimiter! "| ") ; sic
    (#multi_line_scope_only! "dangling_list_like")
  )
)
(
  [
    ":"
    "="
  ]
  .
  (polymorphic_variant_type
    (tag_specification) @append_empty_scoped_softline
    (#scope_id! "dangling_list_like")
  )
)
; Default version relying on local scope only
(
  [
    ":"
    "="
  ]? @do_nothing
  .
  (polymorphic_variant_type
    "["
    .
    "|" @delete
    .
    (tag_specification)
    (#single_line_only!)
  )
)
(
  [
    ":"
    "="
  ]? @do_nothing
  .
  (polymorphic_variant_type
    "["
    .
    "|"? @do_nothing
    .
    (tag_specification) @prepend_delimiter
    (#delimiter! "| ") ; sic
    (#multi_line_only!)
  )
)
(
  [
    ":"
    "="
  ]? @do_nothing
  .
  (polymorphic_variant_type
    [
      "["
      (tag_specification)
    ] @append_empty_softline
  )
)
(
  [
    ":"
    "="
  ]? @do_nothing
  .
  (polymorphic_variant_type
    "[" @append_indent_start
    "]" @prepend_indent_end
  )
)

; Multi-line definitions must have a linebreak after "=" and before "in":
;
; let a =
;   expression
;   in
;   expression
;
(let_expression
  (value_definition
    (_) @append_spaced_softline
    .
  )
  .
  "in" @append_spaced_softline
)
(let_module_expression
  (module_definition)
  .
  "in" @append_spaced_softline
)
(let_open_expression
  (open_module)
  .
  "in" @append_spaced_softline
)
; There are special cases however. We do not want to break lines after "=" when writing
;
; let f = function
;   | Constructor -> expression
;
; or
;
; let f = fun x ->
;   expression
;
(let_binding
  "=" @append_spaced_softline
  .
  [
    ; expressions
    (_
      .
      [
        "("
        "["
      ]
    )
    (list_expression)
    (record_expression)
    (array_expression)
    (function_expression)
    (fun_expression)
  ]* @do_nothing
)

; We want to write multi-line list-like bindings as:
;
; let foo = [
;   bar;
;   baz;
; ]
;
; We use the `dangling_list_like` scope to "attach" the list-like node
; to the symbol preceding it.
(_
  (#scope_id! "dangling_list_like")
  [
    (_)
    "let"
    "in"
    "of"
    "then"
    "else"
    "private"
    "fun"
    "|"
    "%"
    "="
    "+="
    ":"
    "::"
    "<-"
    "->"
    "*"
    ","
    "("
    "["
    "[|"
    "{"
  ] @append_begin_scope
  .
  _? @do_nothing ; Ad hoc hack to prevent anchor from skipping non-named nodes
  .
  (comment)*
  .
  (_
    .
    _? @do_nothing ; Ad hoc hack to prevent anchor from skipping non-named nodes
    .
    [
      "("
      "["
      "[|"
      "{"
    ]
    [
      ")"
      "]"
      "|]"
      "}"
    ] @prepend_end_scope @prepend_end_measuring_scope
  ) @prepend_begin_measuring_scope
)
(_
  (#scope_id! "dangling_list_like")
  [
    (_)
    "let"
    "in"
    "of"
    "then"
    "else"
    "private"
    "fun"
    "|"
    "%"
    "="
    "+="
    ":"
    "::"
    "<-"
    "->"
    "*"
    ","
    "("
    "["
    "[|"
    "{"
  ] @append_begin_scope
  .
  _? @do_nothing ; Ad hoc hack to prevent anchor from skipping non-named nodes
  .
  (comment)*
  .
  (_
    .
    _? @do_nothing ; Ad hoc hack to prevent anchor from skipping non-named nodes
    .
    (_
      .
      _? @do_nothing ; Ad hoc hack to prevent anchor from skipping non-named nodes
      .
      [
        "("
        "["
        "[|"
        "{"
      ]
      [
        ")"
        "]"
        "|]"
        "}"
      ] @prepend_end_scope @prepend_end_measuring_scope
    ) @prepend_begin_measuring_scope
  )
)
(_
  (#scope_id! "dangling_list_like")
  [
    (_)
    "let"
    "in"
    "of"
    "then"
    "else"
    "private"
    "fun"
    "|"
    "%"
    "="
    "+="
    ":"
    "::"
    "<-"
    "->"
    "*"
    ","
    "("
    "["
    "[|"
    "{"
  ] @append_begin_scope
  .
  _? @do_nothing ; Ad hoc hack to prevent anchor from skipping non-named nodes
  .
  (comment)*
  .
  (_
    .
    _? @do_nothing ; Ad hoc hack to prevent anchor from skipping non-named nodes
    .
    (_
      .
      _? @do_nothing ; Ad hoc hack to prevent anchor from skipping non-named nodes
      .
      (_
        .
        _? @do_nothing ; Ad hoc hack to prevent anchor from skipping non-named nodes
        .
        [
          "("
          "["
          "[|"
          "{"
        ]
        [
          ")"
          "]"
          "|]"
          "}"
        ] @prepend_end_scope @prepend_end_measuring_scope
      ) @prepend_begin_measuring_scope
    )
  )
)

; We want to add a line when the regular scope is multi-line,
; But only if the (measured) custom scope is single-line.
; In essence, we want to preserve all of the following three:
;
; let foo = {a = 1; b = 2}
;
; let foo =
;   {a = 1; b = 2}
;
; let foo = {
;   a = 1;
;   b = 2
; }
(_
  [
    (concat_operator)
    "in"
    "of"
    "then"
    "else"
    "private"
    "="
    "+="
    ":"
    "::"
    "<-"
    "*"
  ] @append_spaced_softline @append_indent_start
  .
  _? @do_nothing
  .
  (comment)*
  .
  (_
    .
    [
      "("
      "["
      "[|"
      "{"
    ]
    [
      ")"
      "]"
      "|]"
      "}"
    ] @prepend_indent_end
    .
  )
  .
  "="? @do_nothing ; Abort if we're in a let binding before the `=`
  (#single_line_scope_only! "dangling_list_like")
)
; Do not indent `in` and `->`, because we want to be able to chain them while
; keeping alignment
(_
  [
    "in"
    "->"
  ] @append_spaced_softline
  .
  [
    ; expressions
    (parenthesized_expression)
    (list_expression)
    (record_expression)
    (array_expression)
  ]
  (#single_line_scope_only! "dangling_list_like")
)

; We chose not to add a line break between `=` and the
; `fun` or `function` keywords, but in order to keep the multi-lined-ness
; we must add a softline after the arrow. We need custom scopes to do this,
; since the node which contains the arrow may be single-line.
;
; This turns
;
; let foo =
;   fun x -> x
; in
; bar
;
; into
;
; let foo = fun x ->
;   x
; in
; bar
(let_binding
  "=" @prepend_begin_scope
  .
  (fun_expression
    "->" @append_spaced_scoped_softline
  ) @append_end_scope
  (#scope_id! "fun_definition")
)

; The same as above holds for single-line `function`.
;
; This turns
; let foo =
;   function true -> false | false -> true
; in
; bar
;
; into
;
; let foo = function
;   | true -> false
;   | false -> true
; in
; bar
(let_binding
  "=" @prepend_begin_scope
  .
  (function_expression
    "function" @append_spaced_scoped_softline
  ) @append_end_scope
  (#scope_id! "function_definition")
)
; The following is the general case, which should happen anywhere except in let bindings
(
  "="? @do_nothing
  .
  (function_expression
    "function" @append_spaced_scoped_softline
  ) @prepend_begin_scope @append_end_scope
  (#scope_id! "function_definition")
)
(function_expression
  (match_case)? @do_nothing
  .
  "|" @delete
  .
  (match_case)
  (#single_line_scope_only! "function_definition")
)
(function_expression
  "|"? @do_nothing
  .
  (match_case) @prepend_delimiter
  (#multi_line_scope_only! "function_definition")
  (#delimiter! "| ") ; sic
)
(function_expression
  "|" @prepend_spaced_scoped_softline
  .
  (match_case)
  (#scope_id! "function_definition")
)

(value_definition
  (let_and_operator) @prepend_spaced_softline
)

; There is a large class of terms which should be separated from "=" by a soft line break.
(
  "=" @append_spaced_softline
  .
  [
    (application_expression)
    (class_body_type)
    (if_expression)
    (function_type)
    (let_expression)
    (object_expression)
    (product_expression)
    (sequence_expression)
    (set_expression)
    (typed_expression)
    (unit)
    (value_path)
    (variant_declaration)
  ]
)

; In module signature, each symbol declaration is separated by a softline.
;
; module type Name = sig
;   val function1 : type1
;   val function2 : type2
;  end
;
(signature
  "sig" @append_spaced_softline
  [
    (module_definition)
    (value_specification)
    (type_definition)
    ; start equivalence class
    (include_module)
    (include_module_type)
    ; end equivalence class
  ] @append_spaced_softline
)

; In class definitions and class type definitions, each declaration is separated
; by a softline.

; class foo =
;   object
;     inherit bar
;     val baz = None
;     method qux = None
;   end
;
(object_expression
  "object"
  [
    (inheritance_definition)
    (instance_variable_definition)
    (method_definition)
    (class_initializer)
  ] @append_spaced_softline @prepend_spaced_softline
)

(class_body_type
  "object"
  [
    (inheritance_specification)
    (instance_variable_definition)
    (method_definition)
    (class_initializer)
  ] @append_spaced_softline @prepend_spaced_softline
)

; Move semicolon delimiters just after field declarations,
; before any attributes and comments.
;
; type t =
;   { mutable position : int [@default 0] (* End-of-line comment *);
;   ...
;
; is turned into
;
; type t =
;   {
;     mutable position : int; [@default 0] (* End-of-line comment *)
;     ...
;
(record_declaration
  (field_declaration) @append_delimiter
  .
  [
    (comment)
    (attribute)
  ]*
  .
  ";" @delete
  (#delimiter! ";")
)

; Unfortunately, we can't process all record declarations:
; some are arbitrarily nested within (constructed_type) or (function_type),
; and it is tedious to define the scope in those cases.
(
  [
    ":"
    "="
    "of"
  ]
  (record_declaration
    (field_declaration) @prepend_empty_scoped_softline
    (#scope_id! "dangling_list_like")
  )
)
(
  [
    ":"
    "="
    "of"
  ]
  (record_declaration
    "}" @prepend_empty_scoped_softline
    (#scope_id! "dangling_list_like")
  )
)

; Allow multi-line attributes after field declaratioms, such as:
; type t = {
;   bar: float;
;   foo: bool
;     [@default false] (* a comment *)
;     [@other tag] (* and another one *)
;     [@and again] (* and another one *)
;     [@and again]; (* and a last one *)
; }
(record_declaration
  (#scope_id! "field_declaration")
  [
    (field_declaration)
    (attribute)
    (comment)
  ]? @append_end_scope
  .
  (field_declaration) @prepend_begin_scope
)
(record_declaration
  (#scope_id! "field_declaration")
  [
    (field_declaration)
    (attribute)
    (comment)
  ] @append_end_scope
  .
  "}"
)
(record_declaration
  (attribute) @prepend_indent_start @prepend_spaced_scoped_softline @append_indent_end
  (#scope_id! "field_declaration")
)

; Duplicate the same logic as above for record *expressions*
(record_expression
  (field_expression) @append_delimiter
  .
  [
    (comment)
    (attribute)
  ]*
  .
  ";" @delete
  (#delimiter! ";")
)

(record_expression
  (field_expression) @prepend_empty_scoped_softline
  (#scope_id! "dangling_list_like")
)

(record_expression
  "}" @prepend_empty_scoped_softline
  (#scope_id! "dangling_list_like")
)

(record_expression
  .
  "{" @append_begin_scope
  (#scope_id! "field_expression")
)
(record_expression
  (#scope_id! "field_expression")
  [
    (field_expression)
    (attribute)
    (comment)
  ] @append_end_scope
  .
  (field_expression) @prepend_begin_scope
)
(record_expression
  (#scope_id! "field_expression")
  [
    (field_expression)
    (attribute)
    (comment)
  ] @append_end_scope
  .
  "}"
)
(record_expression
  (attribute) @prepend_indent_start @prepend_spaced_scoped_softline @append_indent_end
  (#scope_id! "field_expression")
)

; Minimized version of the above for list expressions and array expressions

(list_expression
  ";" @append_empty_scoped_softline
  (#scope_id! "dangling_list_like")
)

(array_expression
  ";" @append_empty_scoped_softline
  (#scope_id! "dangling_list_like")
)

; Start an indented block after these
[
  "begin"
  "do"
  "object"
  "sig"
  "struct"
  "then"
] @append_indent_start

; "{" can be used to start quoted strings. Don't indent in that case
(
  "{" @append_indent_start
  .
  (quoted_string_content)* @do_nothing
)

; End the indented block before these
(
  [
    "done"
    "end"
  ] @prepend_indent_end
)

; "}" can be used to end quoted strings. Don't indent in that case
(
  (quoted_string_content)* @do_nothing
  .
  "}" @prepend_indent_end
)

; Only indent after "else" if it's not an "else if" construction
(
  (else_clause
    "else" @append_indent_start
    (if_expression)? @do_nothing
  ) @append_indent_end
)

; End the indented block after these
(
  (then_clause) @append_indent_end
)

; Make an indented block after ":" in typed expressions
;
; (fun _env _md -> assert false) :
;   Env.t -> Parsetree.module_expr -> Typedtree.module_expr * Shape.t
;
(typed_expression
  ":" @append_indent_start
  (_) @append_indent_end
  .
  ; just doing _ above doesn't work, because it matches the final named node as
  ; well as the final non-named node, causing double indentation.
)

(value_specification
  ":" @append_indent_start
  (_) @append_indent_end
  .
)

; Make an indented block after "=" in
; * class bindings
; * class_type bindings
; * instance variable definitions
; * method definitions
; * type bindings

(class_binding
  "=" @append_indent_start
  (_) @append_indent_end
)

(class_type_binding
  "=" @append_indent_start
  (_) @append_indent_end
)

(instance_variable_definition
  "=" @append_indent_start
  (_) @append_indent_end
)

(method_definition
  "=" @append_indent_start
  (_) @append_indent_end
)

; Some more rules for type bindings:
; Don't indent for record types nor polymorphic variant types:
; they are already indented, and we don't process double indentation well enough
(type_binding
  [
    "="
    "+="
  ] @append_indent_start
  .
  [
    (constructed_type
      ; We don't want to capture constructed types that add their own indentation
      "("? @do_nothing
    )
    (function_type)
    (hash_type)
    (object_type)
    (package_type)
    (parenthesized_type)
    (tuple_type)
    (type_constructor_path)
    (type_variable)
    (variant_declaration)
  ] @append_indent_end
  .
  (type_constraint)? @do_nothing
)

(type_binding
  [
    "="
    "+="
  ] @append_indent_start
  .
  [
    (constructed_type)
    (function_type)
    (hash_type)
    (object_type)
    (package_type)
    (parenthesized_type)
    (tuple_type)
    (type_constructor_path)
    (type_variable)
    (variant_declaration)
  ]
  .
  (type_constraint) @append_indent_end
)

(type_binding
  (package_type
    .
    "(" @append_empty_softline @append_indent_start
    ")" @prepend_empty_softline @prepend_indent_end
    .
  ) @prepend_spaced_softline
)

; Consider type constraints to be "out of the block" when deciding
; whether to add a newline between "=" and a constructed type.
; This allows the following to be formatted as it is:
;
; type 'a x = 'a option
;   constraint 'a = ('b,'c)
(type_binding
  [
    "="
    "+="
  ] @prepend_begin_scope
  .
  [
    (constructed_type)
    (function_type)
    (hash_type)
    (object_type)
    (package_type)
    (parenthesized_type)
    (tuple_type)
    (type_constructor_path)
    (type_variable)
    (variant_declaration)
  ] @append_end_scope
  (#scope_id! "type_binding_before_constraint")
)

(type_binding
  [
    "="
    "+="
  ] @append_spaced_scoped_softline
  .
  [
    (constructed_type
      "("? @do_nothing
    )
    (function_type)
    (hash_type)
    (object_type)
    (package_type)
    (parenthesized_type)
    (tuple_type)
    (type_constructor_path)
    (type_variable)
    (variant_declaration)
  ]
  (#scope_id! "type_binding_before_constraint")
)

(type_binding
  (type_constraint) @prepend_spaced_softline
)

; Make an indented block after "of" or ":" in constructor declarations
;
; | Expr_type_clash of
;   Errortrace.unification_error * type_forcing_context option
; | Pattern_type_clash :
;   Errortrace.unification_error * Parsetree.pattern_desc option -> error
;
(constructor_declaration
  [
    "of"
    ":"
  ] @append_indent_start
  (_) @append_indent_end
  .
)

; Make an indented block after the first term in a long if expression
;
; if len < 0
;  || srcoff < 0
;  || srcoff > src.position - len then
;
(if_expression
  (infix_expression
    .
    (_) @append_indent_start
    (_) @append_indent_end
    .
  )
)

; Make an indented block after match arrows.
;
; match lid with
; | Ldot(_, "false") ->
;   Boolean
;
(match_case
  "->" @append_indent_start
  (_) @append_indent_end
  .
)

; Make an indented block after constructor declaration arrows.
;
; Errortrace.unification_error * Parsetree.pattern_desc option ->
;   error
;
(constructor_declaration
  "->" @append_indent_start
  (_) @append_indent_end
  .
)

; Make an indented block where a function/match starts in PPX syntax.
; The special case about function expressions allows the following to be formatted
; as such, instead of having a double indentation in the function:
; let x =
;   [%expr function
;     | false -> 0.
;     | true -> 1.
;   ]
;
; This case has been introduced because of a double indentation in
; https://github.com/topiary/topiary/pull/724
(extension
  .
  "[%" @append_indent_start
  (attribute_payload
    (expression_item
      [
        (function_expression)
        (fun_expression)
      ]? @do_nothing
    )?
  )
  "]" @prepend_indent_end
  .
)
(extension
  "]" @prepend_empty_softline
  .
)

; Indent and add softlines in multiline application expressions, such as
; let _ =
;   long_function
;     long_argument_1
;     long_argument_2
;     long_argument_3
;     long_argument_4
;
; When the last argument is a (parenthesized) function application, end the indentation
; _before_ the application. This allows the following to be formatted as such:
; let () =
;   foo bar (fun x ->
;     something horrible onto x
;   )
; But when the function application minus the last argument is multiline,
; the whole scope still must be indented:
; let () =
;   foo
;     bar
;     (fun x ->
;       x
;     )
;
; Because of these constraints, we must use measuring scopes here: the position of the
; indent_end depends on the multi-line-ness of a subsection of the whole scope.
(application_expression
  .
  (_) @append_indent_start @prepend_begin_scope @prepend_begin_measuring_scope
  (#scope_id! "function_application")
  (_) @append_end_scope
  .
)
; The end of the measuring scope depends on the last argument: if it's a node that
; adds its own indentation, end it before the argument. Otherwise end it after the
; last argument. In that case, it's the same as the regular scope.
(application_expression
  (#scope_id! "function_application")
  (_
    .
    _? @do_nothing ; Ad hoc hack to prevent anchor from skipping non-named nodes
    .
    [
      "("
      "["
      "[|"
      "{"
      (fun_expression)
      (function_expression)
      (parenthesized_expression)
    ]? @do_nothing
  ) @append_end_measuring_scope
  .
)
(application_expression
  (#scope_id! "function_application")
  (_
    .
    _? @do_nothing ; Ad hoc hack to prevent anchor from skipping non-named nodes
    .
    [
      "("
      "["
      "[|"
      "{"
      (fun_expression)
      (function_expression)
      (parenthesized_expression)
    ] @prepend_end_measuring_scope
  )
  .
)
; If the measuring scope is single-line, end indentation _before_ the last node.
; Otherwise, end the indentation after the last node.
(application_expression
  (#multi_line_scope_only! "function_application")
  (_) @append_indent_end
  .
)
(application_expression
  (#single_line_scope_only! "function_application")
  (_) @prepend_indent_end
  .
)
; The node to which we apply append_spaced_scoped_softline will always
; be in both scopes, regular and measuring.
(application_expression
  (_) @append_spaced_scoped_softline
  .
  (_)
  (#scope_id! "function_application")
)

; Indent and allow softlines in multiline function definitions, such as
; let long_function
;     (long_argument_1: int)
;     (long_argument_2: int)
;     (long_argument_3: int)
;     (long_argument_4: int)
;     : int
;   =
;   42
;
; Do not indent if a function expression is being bound:
; the function itself will add the indentation, as in
; let horrible = fun x ->
;   something horrible onto x
(let_binding
  .
  (_) @append_indent_start
  "="
  (_
    ; any node that doesn't add its own indentation
    .
    [
      "fun" ; fun_expression
      "function" ; function_expression
      "[" ; list_expression
      "[|" ; array_expression
      "{" ; record_expression. Unfortunately this also captures quoted strings
      "(" ; parenthesized_expression. Unfortunately this also captures unit
    ]? @do_nothing
  ) @append_indent_end
  .
)
; Special case to still indent quoted strings and unit, despite the rule above
(let_binding
  .
  (_) @append_indent_start
  "="
  [
    (quoted_string)
    (unit)
  ] @append_indent_end
  .
)

(let_binding
  .
  (_) @append_indent_start
  "=" @prepend_indent_end
)
(let_binding
  .
  (_) @prepend_begin_scope
  "=" @append_end_scope
  (#scope_id! "let_binding_before_equal")
)
(let_binding
  (parameter) @prepend_spaced_scoped_softline
  (#scope_id! "let_binding_before_equal")
)
(let_binding
  ":"? @prepend_spaced_scoped_softline
  "=" @prepend_spaced_scoped_softline
  (#scope_id! "let_binding_before_equal")
)

; Indent and allow softlines in anonymous function definitions, such as
; fun
;   (long_argument_1: int)
;   (long_argument_2: int)
;   (long_argument_3: int)
;   (long_argument_4: int) ->
;   ()
;
; The particular interaction with "concat_operator" comes from
; https://github.com/topiary/topiary/pull/723
(
  [
    (concat_operator)
    (rel_operator)
  ]? @do_nothing
  .
  (fun_expression
    .
    "fun" @append_indent_start
    (_) @append_indent_end
    .
  )
)
(fun_expression
  .
  "fun" @append_indent_start
  "->" @prepend_indent_end
)
(fun_expression
  .
  "fun" @prepend_begin_scope
  "->" @append_end_scope
  (#scope_id! "fun_expr_before_arrow")
)
(fun_expression
  (parameter) @prepend_spaced_scoped_softline
  (#scope_id! "fun_expr_before_arrow")
)
(fun_expression
  ":"? @prepend_spaced_scoped_softline
  "->" @prepend_spaced_scoped_softline
  (#scope_id! "fun_expr_before_arrow")
)
(
  [
    (concat_operator)
    (rel_operator)
  ]? @do_nothing
  .
  (function_expression
    .
    "function" @append_indent_start
    (_) @append_indent_end
    .
  )
)

; Indent and allow softlines in tuples and local opens, such as
; let _ =
;   (
;     long_value_1,
;     long_value_2,
;     long_value_3
;   )
;
; When the parenthesized expression contains a function, neither indent
; nor add a softline after the "(".
(parenthesized_expression
  .
  "(" @append_empty_softline @append_indent_start
  [
    (fun_expression)
    (function_expression)
  ]? @do_nothing
  ")" @prepend_indent_end
  .
)
(parenthesized_expression
  ")" @prepend_empty_softline
  .
)
(local_open_expression
  "."
  .
  "(" @append_empty_softline @append_indent_start
  ")" @prepend_indent_end @prepend_empty_softline
  .
)
; Parenthesis are optional when using tuples, so scopes must be tied
; to the `product_expression` inside.
; Product expressions are nested grammar elements, which means that the syntax tree of
; 1, 2, 3
; is
; {Node product_expression}
;   {Node product_expression}
;     {Node number}
;     {Node ,}
;     {Node number}
;   {Node ,}
;   {Node number}
; We only want to define a scope around the outermost `product_expression`,
; which is the one that *isn't* followed by a comma.
(
  (product_expression) @prepend_begin_scope @append_end_scope
  .
  ","? @do_nothing
  (#scope_id! "tuple")
)
(product_expression
  "," @append_spaced_scoped_softline
  (#scope_id! "tuple")
)

; Allow softlines in function and functor types, such as
; type t =
;   a ->
;   (b -> c) ->
;   d ->
;   e
; Function types are nested grammar elements, which means that the syntax tree of
; a -> b -> c
; is
; {Node function_type}
;   {Node type_constructor_path}
;     {Node type_constructor}
;   {Node ->}
;   {Node function_type}
;     {Node type_constructor_path}
;       {Node type_constructor}
;     {Node ->}
;     {Node type_constructor_path}
;       {Node type_constructor}
; We only want to define a scope around the outermost `function_type` node,
; which is the one that *isn't* preceded by an arrow.
(
  "->"? @do_nothing
  .
  (function_type) @prepend_begin_scope @append_end_scope
  (#scope_id! "function_type")
)
(function_type
  "->" @append_spaced_scoped_softline
  (#scope_id! "function_type")
)

(
  "->"? @do_nothing
  .
  (functor_type) @prepend_begin_scope @append_end_scope
  (#scope_id! "functor_type")
)
(functor_type
  "->" @append_spaced_scoped_softline
  (#scope_id! "functor_type")
)

; Allow softlines in boolean infix expressions, such as
; let b =
;   foo
;   || bar
;   || baz

; As above, infix expressions are nested grammar elements, so we must identify the
; top-level one: it is the one that is not preceded by a corresponding infix operator.
(
  (and_operator)? @do_nothing
  .
  (infix_expression
    (and_operator)
  ) @prepend_begin_scope @append_end_scope
  (#scope_id! "and_infix_expression")
)
(infix_expression
  (and_operator) @prepend_spaced_scoped_softline
  (#scope_id! "and_infix_expression")
)

(
  (or_operator)? @do_nothing
  .
  (infix_expression
    (or_operator)
  ) @prepend_begin_scope @append_end_scope
  (#scope_id! "or_infix_expression")
)
(infix_expression
  (or_operator) @prepend_spaced_scoped_softline
  (#scope_id! "or_infix_expression")
)

; When a "rel_operator" (like ">>=") is followed by a function definition,
; do not add a newline
(
  (rel_operator)? @do_nothing
  .
  (infix_expression
    (rel_operator)
  ) @prepend_begin_scope @append_end_scope
  (#scope_id! "rel_infix_expression")
)
(infix_expression
  (rel_operator) @prepend_spaced_scoped_softline
  (fun_expression)? @do_nothing
  (function_expression)? @do_nothing
  (#scope_id! "rel_infix_expression")
)

; Put softline and indented blocks after all other infix operators
(infix_expression
  operator: [
    (pow_operator)
    (mult_operator)
    (add_operator)
    (assign_operator)
  ] @append_spaced_softline @append_indent_start
  right: (_) @append_indent_end
)

; After concat_operator, we want to place a spaced_softline,
; except when a function definition or a list-like structure follows.
(infix_expression
  operator: (concat_operator) @append_spaced_softline
  .
  (array_expression)? @do_nothing
  (list_expression)? @do_nothing
  (parenthesized_expression)? @do_nothing
  (record_expression)? @do_nothing
  (fun_expression)? @do_nothing
  (function_expression)? @do_nothing
)

; Then, we want to indent the expression after a concat_operator
; (with the exception of concat_operator chains, and nodes that add their
; own indentation, like function definitions and lists)
; Ideally like so:
;
; let two =
;   foo @
;   bar @@
;   baz ^^
;   quux
;
; let three =
;   raise @@
;     Exception
;
; let four =
;   run @@ fun x ->
;   something horrible onto x
;
; The particular interaction with "fun_expression" comes from
; https://github.com/topiary/topiary/pull/723
(_
  ; If the parent expression also was a concat_operator, do not indent (see above).
  (concat_operator)? @do_nothing
  (infix_expression
    operator: (concat_operator) @append_indent_start
    (infix_expression
      operator: (concat_operator)
    )? @do_nothing
    (array_expression)? @do_nothing
    (fun_expression)? @do_nothing
    (list_expression)? @do_nothing
    (parenthesized_expression)? @do_nothing
    (record_expression)? @do_nothing
  ) @append_indent_end
)

; The same holds for rel_operator, such as `>>=`
(_
  ; If the parent expression also was a rel_operator, do not indent (see above).
  (rel_operator)? @do_nothing
  (infix_expression
    operator: (rel_operator) @append_indent_start
    (infix_expression
      operator: (rel_operator)
    )? @do_nothing
    (fun_expression)? @do_nothing
  ) @append_indent_end
)

; The following bit of scope sorcery allows the following to be formatted as such:
;
; foo
;   bar
;   baz
;   @@ fun x ->
;   x
;
; While leaving the following as such:
;
; foo bar baz @@ fun x ->
; x
;
; The related issue is https://github.com/topiary/topiary/issues/731
(infix_expression
  (application_expression) @prepend_begin_scope @prepend_begin_measuring_scope
  .
  (concat_operator) @append_end_measuring_scope
  .
  (#scope_id! "dangling_multiline_function")
  [
    (fun_expression)
    (function_expression)
  ] @append_end_scope
)
(infix_expression
  (application_expression)
  .
  (concat_operator) @prepend_spaced_softline @prepend_indent_start
  .
  (#multi_line_scope_only! "dangling_multiline_function")
  [
    (fun_expression)
    (function_expression)
  ] @append_indent_end
)

; The following allows
;
; somefun @@
;   fun x -> body
;
; to be formatted as
;
; somefun @@ fun x ->
; body
(infix_expression
  (#scope_id! "relocate_dangling_function_line_break")
  (concat_operator) @append_begin_scope @append_begin_measuring_scope
  .
  (fun_expression
    "fun" @prepend_end_measuring_scope
    "->" @append_end_scope
  )
)
(infix_expression
  (#multi_line_scope_only! "relocate_dangling_function_line_break")
  (concat_operator)
  .
  (fun_expression
    "->" @append_hardline
  )
)

; Allow softlines in sequences and ppx sequences, such as
; let b =
;   foo;
;   bar;
;   baz
; As above, sequences are nested grammar elements, so we must identify the
; top-level one: it is the one that is not preceded by a ";" (or ";%foo" for ppx sequences).
(
  ";"? @do_nothing
  (sequence_expression
    ";"
  ) @prepend_begin_scope @append_end_scope
  (#scope_id! "sequence_expression")
)
(sequence_expression
  ";" @append_spaced_scoped_softline
  .
  "%"? @do_nothing
  (#scope_id! "sequence_expression")
)
(sequence_expression
  ";"
  .
  "%"
  .
  (attribute_id) @append_spaced_scoped_softline
  (#scope_id! "sequence_expression")
)

; Allow softlines in or patterns in matches, such as
; match foo with
; | A | B | C
; | D | E | F -> bar
; As above, or patterns are nested grammar elements, so we must identify the
; top-level one: it is the one that is directly below a (match_case) node.

(match_case
  (or_pattern) @prepend_begin_scope @append_end_scope
  (#scope_id! "top_level_or_pattern")
)
(match_case
  (alias_pattern) @prepend_begin_scope @append_end_scope
  (#scope_id! "top_level_or_pattern")
)

(or_pattern
  "|" @prepend_hardline
  (#multi_line_scope_only! "top_level_or_pattern")
)

; Indent and add softlines in lists and arrays, such as
; let _ = [
;   long_value_1;
;   long_value_2;
;   long_value_3;
; ]
(
  (
    [
      (concat_operator)
      "in"
      "then"
      "else"
      "="
      "+="
      ":"
      "::"
      "->"
      "*"
    ]
  )
  .
  (list_expression
    .
    "[" @append_indent_start @append_empty_softline
    "]" @prepend_indent_end @prepend_empty_softline
    .
  )
)
(application_expression
  (list_expression
    .
    "[" @append_indent_start @append_empty_softline
    "]" @prepend_indent_end @prepend_empty_softline
    .
  )
  .
)
(list_pattern
  .
  "[" @append_indent_start @append_empty_softline
  "]" @prepend_indent_end @prepend_empty_softline
  .
)
(list_pattern
  .
  "["
  ";" @append_empty_softline
  "]"
)
(record_pattern
  .
  "{" @append_empty_softline
  "}" @prepend_empty_softline
  .
)
(record_pattern
  .
  "{"
  ";" @append_empty_softline
  "}"
)
(array_pattern
  .
  "[|"
  ";" @append_empty_softline
  "|]"
)

; Unfortunately, we can't process all polymorphic variant types:
; some are arbitrarily nested within (constructed_type) or (function_type),
; and it is tedious to define the scope in those cases.
(
  [
    "="
    ":"
  ]
  (polymorphic_variant_type
    .
    "[" @append_indent_start @append_empty_softline
    "]" @prepend_indent_end @prepend_empty_softline
    .
  )
)
; Same for parenthesized constructed types, and record declarations.
; This is a best effort situation.
(
  [
    "="
    ":"
  ]
  (constructed_type
    .
    "(" @append_indent_start @append_empty_softline
    ")" @prepend_indent_end @prepend_empty_softline
  )
)
(
  [
    "="
    ":"
  ]
  (constructed_type
    .
    "("
    "," @append_spaced_softline
    ")"
  )
)
(
  [
    "="
    ":"
  ] @append_empty_softline
  (constructed_type
    "("
    ")"
  )
  "="? @do_nothing ; desist in let binding before equal
  (#single_line_scope_only! "dangling_list_like")
)

(array_expression
  .
  "[|" @append_indent_start @append_empty_softline
  "|]" @prepend_indent_end @prepend_empty_softline
  .
)

(array_pattern
  .
  "[|" @append_indent_start @append_empty_softline
  "|]" @prepend_indent_end @prepend_empty_softline
  .
)

; Allow softlines and indentation in functor definitions with many arguments, such as
; module Lift
;   (Credit: module type of CreditSignature)
;   (Dance: module type of DanceSignature)
;   (Tune: module type of TuneSignature)
;   (Version: module type of VersionSignature)
; = struct
;   let foo = x
; end
(module_binding
  (module_name) @append_indent_start @prepend_begin_scope
  [
    "="
    ":"
  ] @prepend_empty_scoped_softline @prepend_indent_end @append_end_scope
  (#scope_id! "module_binding_before_equal")
)
; if a module binding has no equal sign and isn't just a signature, everything enters the scope
(module_binding
  (#scope_id! "module_binding_before_equal")
  (module_name) @append_indent_start @prepend_begin_scope
  "="? @do_nothing
  (signature)? @do_nothing
) @append_indent_end @append_end_scope
(module_binding
  (module_name) @append_empty_scoped_softline
  (module_parameter) @prepend_spaced_scoped_softline
  (#scope_id! "module_binding_before_equal")
)

; Try block formatting
; A soft linebreak after the "try" (potentially "try%ppx") and one after the "with".
(try_expression
  "try" @append_spaced_softline @append_indent_start
  .
  "%"* @do_nothing
)
(try_expression
  "try"
  .
  "%"
  .
  (attribute_id) @append_spaced_softline @append_indent_start
)
(try_expression
  "with" @prepend_indent_end @prepend_spaced_softline @append_indent_start
  (_) @append_indent_end
  .
)

; Formatting set expressions
(set_expression
  "<-" @append_spaced_softline @append_indent_start
) @append_indent_end

; Antispaces for brackets and parentheses
(
  [
    "["
    "[|"
    "{"
  ] @append_antispace
)
(
  [
    "]"
    "|]"
    "}"
  ] @prepend_antispace
)

; We must be cautious when surrounding `mult_operator` with parentheses, lest we mess
; with comments:
; `val ( * ): x -> y -> x` must not be formatted into `val (*): x -> y -> x`
(
  "(" @append_antispace
  .
  (mult_operator)? @do_nothing
)
(
  (mult_operator)? @do_nothing
  .
  ")" @prepend_antispace
)

; Formatting typed patterns in function arguments, e.g.
; let foo
;   (bar :
;     int ->
;     string ->
;     unit
;   )
;   ~(baz :
;     int ->
;     string ->
;     unit
;   )
;   =
;   bar baz
(typed_pattern
  .
  "("
  ":" @append_spaced_softline @append_indent_start
  ")" @prepend_indent_end @prepend_empty_softline
  .
)
(parameter
  "~"
  .
  "("
  ":" @append_spaced_softline @append_indent_start
  ")" @prepend_indent_end @prepend_empty_softline
  .
)

; Input softlines before and after all comments. This means that the input
; decides if a comment should have line breaks before or after. But don't put a
; softline directly in front of commas or semicolons.

(comment) @prepend_input_softline

(
  (comment) @append_input_softline
  .
  ["," ";"]* @do_nothing
)
