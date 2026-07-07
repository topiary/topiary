; Configuration
(#language! ocaml)

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
; Multi-
; line comment
    (character)
    (quoted_string)
    (signed_number)
    (string)
  ]
) @leaf

; line number directives must be alone on their line, and can't be indented
(line_number_directive) @single_line_no_indent

; Allow blank line before
[
  (class_definition)
  (class_type_definition)
  (comment)
  (exception_definition)
  (external)
  (floating_attribute)
  ; equivalence class
  [
    (include_module)
    (include_module_type)
  ]
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

; In a definition including several mutually recursive functions,
; one can skip a line before each of them.
(value_definition
  "and" @allow_blank_line_before
)
(value_definition
  (and_operator) @allow_blank_line_before
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
  ] @append_hardline
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
    ; equivalence class
    (include_module)
    (include_module_type)
  ] @append_hardline
  .
  "end"? @do_nothing
)
(structure
  "struct"
  .
  [
    ; equivalence class
    (include_module)
    (include_module_type)
  ] @append_spaced_softline
  .
  "end"
)

; Consecutive definitions must be separated by line breaks
(
  [
    (value_definition)
    (value_specification)
  ] @append_hardline
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
; A space is put after, and before (except just after an open parenthesis).
(
  [
    "and"
    "as"
    "assert"
    "class"
    "constraint"
    "downto"
    "else"
    "exception"
    "external"
    (floating_attribute)
    "for"
    "if"
    "in"
    "include"
    (infix_operator)
    "inherit"
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
    "||"
    "->"
    "<-"
    "{"
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

; Those keywords are not expected to come right after an open parenthesis.
[
    "as"
    "constraint"
    "do"
    "done"
    "downto"
    "else"
    "in"
    "nonrec"
    "of"
    "rec"
    "then"
    "to"
    "virtual"
    "when"
    "with"
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
  (and_operator) @append_space
  .
  ")"* @do_nothing
)
(
  (let_operator) @append_space
  .
  ")"* @do_nothing
)

; For those queries, we should not have multiple queries,
; however, due to a known bug in tree-sitter queries
; https://github.com/tree-sitter/tree-sitter/issues/1811
; using an alternative after the starred parenthesis does not work as intended.
;
(
  "("* @do_nothing
  .
  "assert" @prepend_space
)
(
  "("* @do_nothing
  .
  (attribute) @prepend_space
)
(
  "("* @do_nothing
  .
  "begin" @prepend_space
)
(
  "("* @do_nothing
  .
  "class" @prepend_space
)
(
  "("* @do_nothing
  .
  "exception" @prepend_space
)
(
  "("* @do_nothing
  .
  "external" @prepend_space
)
(
  "("* @do_nothing
  .
  (floating_attribute) @prepend_space
)
(
  "("* @do_nothing
  .
  "for" @prepend_space
)
(
  "("* @do_nothing
  .
  "include" @prepend_space
)
(
  "("* @do_nothing
  .
  (infix_operator) @prepend_space
)
(
  "("* @do_nothing
  .
  "inherit" @prepend_space
)
(
  "("* @do_nothing
  .
  (item_attribute) @prepend_space
)
(
  "("* @do_nothing
  .
  "let" @prepend_space
)
(
  "("* @do_nothing
  .
  "method" @prepend_space
)
(
  "("* @do_nothing
  .
  "module" @prepend_space
)
(
  "("* @do_nothing
  .
  (module_parameter) @prepend_space
)
(
  "("* @do_nothing
  .
  "mutable" @prepend_space
)
(
  "("* @do_nothing
  .
  "new" @prepend_space
)
(
  "("* @do_nothing
  .
  "object" @prepend_space
)
(
  "("* @do_nothing
  .
  "open" @prepend_space
)
(
  "("* @do_nothing
  .
  (parameter) @prepend_space
)
(
  "("* @do_nothing
  .
  "private" @prepend_space
)
(
  "("* @do_nothing
  .
  "sig" @prepend_space
)
(
  "("* @do_nothing
  .
  "try" @prepend_space
)
(
  "("* @do_nothing
  .
  "type" @prepend_space
)
(
  "("* @do_nothing
  .
  "val" @prepend_space
)
(
  "("* @do_nothing
  .
  "while" @prepend_space
)
(
  "("* @do_nothing
  .
  "*" @prepend_space
)
(
  "("* @do_nothing
  .
  "=" @prepend_space
)
(
  "("* @do_nothing
  .
  "||" @prepend_space
)
(
  "("* @do_nothing
  .
  "}" @prepend_space
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
    ; equivalence class
    [
      (extended_module_path)
      (module_path)
      (module_type_path)
    ]
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
    ; equivalence class
    [
      (extended_module_path)
      (module_path)
      (module_type_path)
    ]
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
    "in"
    "of"
    "struct"
    "then"
    "with"
    "{"
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
    "in"
    "of"
    "struct"
    "then"
    "with"
    "{"
    ";"
  ]
  .
  "%"
  .
  (attribute_id) @append_spaced_softline
  .
  (comment)* @do_nothing
)

(type_binding
  (type_constraint) @prepend_spaced_softline
)

; only add softlines after "else" if it's not part of an "else if" construction
(
  "else" @append_spaced_softline
  .
  [
    (comment)
    (if_expression)
    "%"
  ]? @do_nothing
)

(
  "else"
  .
  "%"
  .
  (attribute_id) @append_spaced_softline
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
  (infix_operator)
  (item_attribute)
  "*"
  "|"
  "}"
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
(
  "|"* @do_nothing
  .
  (match_case) @prepend_spaced_softline
)

; Multi-line definitions must have a linebreak after "=" and before "in":
;
; let a =
;   expression
;   in
;   expression
;
(
  (value_definition
    (_) @append_spaced_softline
    .
  )
  .
  "in"
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
    (function_expression)
    (fun_expression)
  ]* @do_nothing
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
;   true -> false
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
(parenthesized_expression
  (function_expression) @prepend_begin_scope @append_end_scope
  (#scope_id! "function_definition")
)
(function_expression
  "|"* @do_nothing
  .
  (match_case) @prepend_spaced_scoped_softline
  (#scope_id! "function_definition")
)
(function_expression
  "|"* @prepend_spaced_scoped_softline
  .
  (match_case)
  (#scope_id! "function_definition")
)

(value_definition
  (and_operator) @prepend_spaced_softline
)

; There is a large class of terms which should be separated from "=" by a soft line break.
(
  "=" @append_spaced_softline
  .
  [
    (application_expression)
    (class_body_type)
    (constructed_type)
    (if_expression)
    (function_type)
    (let_expression)
    (object_expression)
    (product_expression)
    (record_expression)
    (sequence_expression)
    (set_expression)
    (typed_expression)
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
    (value_specification)
    (type_definition)
    ; equivalence class
    [
      (include_module)
      (include_module_type)
    ]
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
  ] @append_spaced_softline @prepend_spaced_softline
)

(class_body_type
  "object"
  [
    (inheritance_specification)
    (instance_variable_definition)
    (method_definition)
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

(record_declaration
  (field_declaration) @prepend_spaced_softline
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
  (field_expression) @prepend_spaced_softline
)

(record_expression
  (#scope_id! "field_expression")
  [
    (field_expression)
    (attribute)
    (comment)
  ]? @append_end_scope
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

; Don't indent for record types nor polymorphic variant types:
; they are already indented, and we don't process double indentation well enough
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
    (parenthesized_type)
    (tuple_type)
    (type_constructor_path)
    (type_variable)
    (variant_declaration)
  ]
  .
  (type_constraint) @append_indent_end
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

; Make an indented block where a function type arrow starts. Only for the root
; level, not for each arrow.
;
; (?used_slot:bool ref ->
;   Longident.t loc ->
;   Path.t * Env.t)
;
(constructed_type
  (function_type
    "->" @append_indent_start
    (_) @append_indent_end
    .
  )
)

; Make an indented block where a function/match starts in PPX syntax.
(extension
  "[%" @append_indent_start
  "]" @prepend_indent_end @prepend_empty_softline
)

; Indent and add softlines in multiline application expressions, such as
; let _ =
;   long_function
;     long_argument_1
;     long_argument_2
;     long_argument_3
;     long_argument_4
(application_expression
  .
  (_) @append_indent_start
  (_) @append_indent_end
  .
)
(application_expression
  (_) @append_spaced_softline
  .
  (_)
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
(let_binding
  .
  (_) @append_indent_start @append_indent_start
  "=" @prepend_indent_end
  (_) @append_indent_end
  .
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
(fun_expression
  .
  "fun" @append_indent_start @append_indent_start
  "->" @prepend_indent_end
  (_) @append_indent_end
  .
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

; Indent and allow softlines in tuples, such as
; let _ =
;   (
;     long_value_1,
;     long_value_2,
;     long_value_3
;   )
(parenthesized_expression
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

; Allow softlines in function types, such as
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

; Allow softlines in infix expressions, such as
; let b =
;   foo
;   || bar
;   || baz

; As above, infix expressions are nested grammar elements, so we must identify the
; top-level one: it is the one that is not preceded by an infix operator.
; We only consider the common logic operators, as not to mess with arithmetic expressions
(
  (infix_operator
    [
      "||"
      "&&"
    ]
  )? @do_nothing
  .
  (infix_expression) @prepend_begin_scope @append_end_scope
  (#scope_id! "infix_expression")
)
(infix_expression
  (infix_operator
    [
      "||"
      "&&"
    ]
  ) @prepend_spaced_scoped_softline
  (#scope_id! "infix_expression")
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
  .
  (sequence_expression
    .
    _
    .
    ";"
    .
    "%"? @do_nothing
  ) @prepend_begin_scope @append_end_scope
  (#scope_id! "sequence_expression")
)
(sequence_expression
  ";" @append_spaced_scoped_softline
  (#scope_id! "sequence_expression")
)

(
  ";"?
  .
  "%"? @do_nothing
  .
  (attribute_id)?
  .
  (sequence_expression
    .
    _
    .
    ";"
    .
    "%"
  ) @prepend_begin_scope @append_end_scope
  (#scope_id! "ppx_sequence_expression")
)
(sequence_expression
  ";"
  .
  "%"
  .
  (attribute_id) @append_spaced_scoped_softline
  (#scope_id! "ppx_sequence_expression")
)

; Indent and add softlines in lists and arrays, such as
; let _ =
;   [
;     long_value_1;
;     long_value_2;
;     long_value_3;
;   ]
(list_expression
  .
  "[" @append_indent_start @append_empty_softline
  "]" @prepend_indent_end @prepend_empty_softline
  .
)

(list_pattern
  .
  "[" @append_indent_start @append_empty_softline
  "]" @prepend_indent_end @prepend_empty_softline
  .
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
  "=" @prepend_empty_scoped_softline @prepend_indent_end @append_end_scope
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

; Input softlines before and after all comments. This means that the input
; decides if a comment should have line breaks before or after. But don't put a
; softline directly in front of commas or semicolons.

(comment) @prepend_input_softline

(
  (comment) @append_input_softline
  .
  [ "," ";" ]* @do_nothing
)

; ================================================================================
; Predicate parameter newline separatation and indentation
; ================================================================================

(#any-of? @function.builtin @foo "union" "difference" "intersection"
"circle")

(#set! 
injection.language
"doxygen")
