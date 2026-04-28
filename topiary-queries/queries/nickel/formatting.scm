;; General Spacing

; The following nodes in our source text should not be formatted
[
  (static_string)
  (str_chunks_single)
  (str_chunks_multi)
  (builtin)
  (quoted_enum_tag)
] @leaf

; Allow a blank line before the following nodes
[
  (comment)
  (field_decl)
  (last_field)
] @allow_blank_line_before

; Allow blank lines to appear between a let bind and its result
(let_expr
  (let_in_block)
  .
  (term) @allow_blank_line_before
)

; Surround with spaces: keywords, operators, annotation markers
(
  [
    "if"
    "then"
    "else"
    "forall"
    "in"
    "let"
    "match"
    "null"
    "true"
    "false"
    "fun"
    "import"
    "default"
    "doc"
    "force"
    "optional"
    "priority"
    "or"
    "include"
    "|"
    ":"
    "?"
    "="
    "rec"
    "Array"
    "Dyn"
    "Number"
    "Bool"
    "String"
    "->"
    "=>"
    ; Infix operators
    "++"
    "@"
    "*"
    "/"
    "%"
    "+"
    "-"
    "!"
    "&"
    "|>"
    "<"
    "<="
    ">"
    ">="
    "=="
    "!="
    "&&"
    "||"
  ] @prepend_space @append_space
)

; Don't insert spaces before the following delimiters
;
; NOTE This will destroy the space in a polymorphic record tail. For
; example: forall. { x: Number; a } -> {; a}
;
; WARNING We don't include "." as it is very common for it to appear in
; string interpolation, for record field access, which will manifest the
; bug documented in Issue #395. The remaining delimiters in this
; alternation can also appear in such contexts, but they're much less
; likely; i.e., this is a trade-off, to avoid over-complicating the
; formatting rules.
[
  ","
  ";"
] @prepend_antispace

; Parentheses
;
; We don't insert spaces immediately after (resp. before) an opening (resp.
; closing) parenthesis.
;
; In a multi-line context, we handle indentation in the same way as for record
; field definitions. If the content is already a form which will add its own new
; line and indentation, we don't have to do anything (function definitions,
; record literals, etc.). Otherwise, say for a long multi-line boolean
; expression, we do add a new (empty soft)line and indent the content.
;
; We always put the closing parenthesis on a new line in a multi-line context,
; because the last line of the content can end up with an arbitrarily deep
; indentation. It's better to visually align the opening and closing
; parentheses, in a way that doesn't depend on what's inside.
(atom
  .
  "(" @append_empty_softline @append_indent_start
  (uni_term
    .
    [
      ; There is scope for factoring these patterns with
      ; embedded alternations. Keeping them separate is probably more
      ; efficient to process and certainly easier to read.

      ; Record literals
      (infix_expr . (applicative . (record_operand . (atom . (uni_record)))))

      ; Array literals
      (infix_expr . (applicative . (record_operand . (atom . "["))))

      ; Enum literals
      (infix_expr . (applicative . (record_operand . (atom . (type_atom . "[|")))))

      ; Parentheticals
      (infix_expr . (applicative . (record_operand . (atom . "("))))

      ; Function declarations
      (fun_expr)

      ; Match statements
      (infix_expr . (applicative . (match_expr)))

      ; Multi-line and symbolic strings
      (infix_expr . (applicative . (record_operand . (atom . (str_chunks)))))
    ]
  )? @do_nothing
  ")" @prepend_indent_end
  .
)

(atom
  .
  "(" @append_antispace
  ")" @prepend_antispace @prepend_empty_softline
  .
)

; Don't insert spaces between infix operators and their operand
(infix_expr
  .
  [
    "-"
    (infix_u_op_5 "!")
  ] @append_antispace
  .
  (infix_expr)
  .
)

; Flow a chain of infix expressions over new lines, in a multi-line
; context. Note that we _don't_ want this to happen for comparison
; operators, which fall under nodes (infix_b_op_7) and (infix_b_op_8).
(uni_term
  (#scope_id! "infix_chain")
  [
    (infix_expr)
    (annotated_infix_expr)
  ] @prepend_begin_scope
) @append_end_scope

(infix_expr
  (#scope_id! "infix_chain")
  (infix_expr)
  .
  [
    (infix_b_op_2) ; ++ and @
    (infix_b_op_3) ; *, / and %
    (infix_b_op_4) ; + and -
    (infix_b_op_6) ; & and |>
    (infix_lazy_b_op_9) ; &&
    (infix_lazy_b_op_10) ; ||
  ] @prepend_spaced_scoped_softline
  .
  (infix_expr)
)

; Surround all polymorphic type variables with spaces
(forall
  (ident) @prepend_space
)

; Insert a space between the enum tag and the argument of an enum variant
(enum_variant
  (enum_tag) @append_space
)

; Insert a space between the enum tag and the argument of an enum variant
; pattern
(enum_variant_pattern
  (enum_tag) @append_space
  (pattern_fun)
)

;; Comments

(comment) @prepend_input_softline @append_hardline

;; Bound Expressions
; i.e., Let expressions and record fields

; The default, multi-line behaviour for the RHS of a bound expression is
; for it to start an indentation block on a new line. However, the
; following idiomatic exceptions should not behave in this way:
;
; * Record literals                   { ... }
; * Array literals                    [ ... ]
; * Enum literals                     [| ... |]
; * Parentheticals                    ( ... )
; * Function declarations             fun ... => ...
; * Match statements                  match { ... }
; * Multi-line and symbolic strings   m%"..."% / xxx-s%"..."%
;
; These should remain in-line. Note that this is true for the right-hand side of
; record patterns as well, but patterns aren't terms, so they aren't subject to
; the general multi-line behavior to begin with. In other words, the right-hand
; side of field patterns behave as the exceptions mentioned above, but we don't
; need any special query to do so.

(_
  (#scope_id! "bound_rhs")
  "=" @prepend_begin_scope
  .
  [
    (term)
    (pattern)
  ] @append_end_scope
)

(_
  (#scope_id! "bound_rhs")
  "=" @append_spaced_scoped_softline @append_indent_start
  .
  (term
    .
    (uni_term
      .
      [
        ; There is scope for factoring these patterns with
        ; embedded alternations. Keeping them separate is probably more
        ; efficient to process and certainly easier to read.

        ; Record literals
        (infix_expr . (applicative . (record_operand . (atom . (uni_record)))))

        ; Array literals
        (infix_expr . (applicative . (record_operand . (atom . "["))))

        ; Enum literals
        (infix_expr . (applicative . (record_operand . (atom . (type_atom . "[|")))))

        ; Parentheticals
        (infix_expr . (applicative . (record_operand . (atom . "("))))

        ; Function declarations
        (fun_expr)

        ; Match statements
        (infix_expr . (applicative . (match_expr)))

        ; Multi-line and symbolic strings
        (infix_expr . (applicative . (record_operand . (atom . (str_chunks)))))
      ]? @do_nothing
    )
  ) @append_indent_end
)

; If the RHS starts with a comment, which itself is followed by a hard
; line, then we apply the normal indent block formatting in a multi-line
; context (i.e., no exceptions)
(_
  "=" @append_indent_start
  .
  (comment)
  (term) @append_indent_end
)

; A let expression looks like:
;
;   let [rec] IDENT = EXPR in EXPR
;
; or
;
;   let [rec]
;     IDENT = EXPR,
;     IDENT = EXPR,
;   in
;   EXPR
;
; The formatting for the bound expression is handled by the above rules,
; which also apply to record field values. The "in" should appear on a
; new line, if the entire let expression is multi-line. The result
; expression (i.e., after the "in") should appear on an new line, if
; that is multi-line. We don't start an indentation block for the result
; expression, to avoid long diagonals in a series of let expressions
; (which is idiomatic).

; A let block containing multiple bindings. If this is multiline, the first
; binding should appear on a new line, and the bindings should be indented.
(let_expr
  (#scope_id! "let_result")
  (let_in_block
    "let"
    .
    ; Prepend before the first binding instead of appending after the "let",
    ; so that in the case of a "let rec" the line break goes after the "rec".
    (let_binding) @prepend_spaced_softline @prepend_indent_start
    (let_binding)
    "in" @prepend_indent_end @prepend_begin_scope @prepend_spaced_softline
  )
  (term) @append_end_scope
)

; A let with a single binding. The binding should be on the same line as the "let".
(let_expr
  (#scope_id! "let_result")
  (let_in_block
    "let"
    .
    (let_binding)
    .
    "in" @prepend_begin_scope @prepend_spaced_softline
  )
  (term) @append_end_scope
)

; When binding multiple values in a let block, allow new lines between the bindings.
(let_expr
  (#scope_id! "let_result")
  (let_in_block
    ("," @append_spaced_softline)
  )
)

(let_expr
  (#scope_id! "let_result")
  (term) @prepend_spaced_scoped_softline
)

;; Annotations

; Start a scope from the node preceding the annotations. This scope is used
; exclusively to lay out annotations on multiple lines. The rule checks if the
; annotations were intended to be on newlines as in:
;
; id
;   | a -> a
;
; which, without the annotations scope, would consider the annotations to be a
; single line node and format it as such:
;
; id | a -> a
(
  (#scope_id! "annotations")
  (_) @append_begin_scope
  .
  (annot) @append_end_scope
)

; Put each annotation on a new line, in a multi-line context.
(annot
  (#scope_id! "annotations")
  (annot_atom) @prepend_spaced_scoped_softline
)

; Start a new scope for annotations, used to properly indent any content coming
; after the annotations. We use Topiary's measuring scope feature: the
; multi-lineness of this scope is entirely decided by the annotations only, but
; what we want to affect is larger, namely the content of the binding coming
; after the `=`.
(
  (#scope_id! "annotations_with_content")
  (_) @append_begin_scope @append_begin_measuring_scope
  .
  (annot) @append_end_measuring_scope
  "="
  (term) @append_end_scope
)

; Add a new line before the last annotation and the following equal sign, when
; the annotations are multi-line.
;
; Indeed, we want to have the following formatting (multi-line):
;
; let foo
;   | Array Number
;   | doc "hello"
;   = [
;     1,
;     2,
;   ]
; in ...
;
; But (single-line):
;
; let foo | Array Number = [
;   1,
;   2,
; ]
; in ...
(
  (#scope_id! "annotations_with_content")
  (annot) @append_spaced_scoped_softline
  .
  "="
)

; Indent the annotations with respect to the identifier they annotate.
(
  (annot) @prepend_indent_start @append_indent_end
)

; Indent the bound expression of a let-binding (or a field definition) in
; presence of multi-line annotations. That's where we use the measuring scope of
; "annotations_with_content": we want this rule to fire only when (annot) is
; multi-line, regardless of the multi-lineness of the ("=" (term)) part.
(_
  (#multi_line_scope_only! "annotations_with_content")
  (annot) @append_indent_start
  "="
  (term) @append_indent_end
)

; Break a multi-line polymorphic type annotation after the type
; variables, starting an indentation block
(forall
  "." @append_spaced_softline @append_indent_start
) @append_indent_end

;; Functions

; Start a function's definition on a new line, in a multi-line context.
; This also defines an indentation block.
(fun_expr
  (#scope_id! "function_definition")
  "=>" @prepend_begin_scope @append_indent_start
) @append_indent_end @append_end_scope

(fun_expr
  (#scope_id! "function_definition")
  (term) @prepend_spaced_scoped_softline
)

(fun_expr
  (pattern_fun) @append_space
)

; Function application (and similar: type applications, enum variants, etc.)

; The applicative operator is a space
;
; In in a multi-line context, we'd like the operands to start on their own line,
; each indented.
;
; In the case of unary applications, we always lay out the function application
; on one line (the argument might still span multiple lines, but the separation
; between the function and the operand is a space), and we don't add
; indentation. Doing otherwise would often add unnecessary indentation when the
; argument is e.g. a function, a record or an array literal, etc.

; The multi-line character of an application depends on both the function and
; each argument, so we crate a scope accordingly.
(infix_expr
  (#scope_id! "applicative_chain")
  (applicative) @prepend_begin_scope
) @append_end_scope

; In the the multi-ary application case, we add a softline before each argument,
; and we indent it.
;
; Note that this pattern won't match the very last argument of an applicative
; chain (the last `t2` of the rule), which needs to be handled separately.
(
  (#scope_id! "applicative_chain")
  (applicative
    t1: (applicative
      t1: (applicative)
      t2: (_) @prepend_indent_start @prepend_spaced_scoped_softline @append_indent_end
    )
    t2: (_)
  )
)

; Missing case of the previous rule to indent the very last argument of a
; multi-ary application
(infix_expr
  (#scope_id! "applicative_chain")
  (applicative
    t1: (applicative
      t1: _
      t2: _
    )
    t2: (_) @prepend_indent_start @prepend_spaced_scoped_softline @append_indent_end
  )
)

; This adds a space before any argument of an application.
;
; In the multi-ary case, this space seems redundant, but it's not an issue as
; Topiary will just absorb it in the scoped softline which will still result in
; a softline.
;
; In the unary case, this is adding the required space between the function and
; its argument. Put differently, this rule handles the unary application case,
; and albeit it does also match multi-ary applications, it doesn't have any
; additional effect in that case.
(applicative
  t1: _
  t2: (_) @prepend_space
)

;; Patterns and match branches

; Define a separate scope for the body of a match case. As we have a separate
; scope for the right-hand side of an "=" (i.e. for bound expressions), we want
; a separate scope for the right-hand side of "=>".
(match_branch
  (#scope_id! "branch_body")
  (pattern)
  "=>" @prepend_begin_scope
  (term) @append_end_scope
)

; Flow multi-line match cases into an indented block after the =>
(match_branch
  (#scope_id! "branch_body")
  "=>" @append_spaced_scoped_softline @append_indent_start
) @append_indent_end

; Add indentation to the condition of pattern guards
(match_branch
  (pattern_guard
    "if" @append_indent_start
    (term) @append_indent_end
  )
)

; Flow each or-branch of an or-pattern on a separate line when they're the
; top-level construct of the pattern
(match_branch
  (pattern
    (or_pattern
      (or_pattern_unparens
        "or" @prepend_spaced_softline
      )
    )
  )
)

;; Conditionals

; if...then...else expressions can either be single or multi-line. In a
; multi-line context, they will be formatted like so:
;
;   if CONDITION then
;     TRUE_TERM
;   else
;     FALSE_TERM
;
; NOTE If the FALSE_TERM is another if...then...else expression, in a
; multi-line context, then the indentation block is "cancelled" to
; create the illusion of an "else if" term:
;
;   if CONDITION1 then
;     TRUE_TERM1
;   else if CONDITION2 then
;     TRUE_TERM2
;   else
;     FALSE_TERM2
;
; This style has precedent from the manually formatted stdlib. (An
; alternative style is to give the "then" token its own line.)
(ite_expr
  "then" @append_spaced_softline @append_indent_start
  "else" @prepend_indent_end @prepend_spaced_softline
)

(ite_expr
  "else" @append_spaced_softline @append_indent_start
  t2: (term
    ; Don't apply formatting if an "else" is followed by an "if"
    (uni_term (ite_expr))? @do_nothing
  ) @append_indent_end
)

;; Container Types
; i.e., Arrays, records (and dictionaries, vicariously) and enums

; We don't want to add spaces/new lines in empty records, so the
; following query only matches if a named node exists within the record
; NOTE This rule also applies to record patterns
(_
  (#scope_id! "container")
  .
  "{" @append_spaced_softline @append_indent_start @prepend_begin_scope
  (_)
  "}" @prepend_indent_end @prepend_spaced_softline @append_end_scope
  .
)

; Unlike records, arrays should never have internal spacing, similar to
; parentheticals. (This is a conscious choice by the Nickel team; see
; Issue #407.)
; NOTE This rule also applies to array patterns and include lists
(_
  (#scope_id! "container")
  .
  "[" @append_empty_softline @append_indent_start @prepend_begin_scope
  (_)
  "]" @prepend_indent_end @prepend_empty_softline @append_end_scope
  .
)

; It doesn't really make sense for an enum to
; have no members, so we ignore that case
(_
  (#scope_id! "container")
  .
  "[|" @append_spaced_softline @append_indent_start @prepend_begin_scope
  "|]" @prepend_indent_end @prepend_spaced_softline @append_end_scope
  .
)

; Allow newlines after the comma (or semicolon) following a container
; element.
(
  (#scope_id! "container")
  [
    (field_decl)
    (last_field)
    (field_pattern)
    (last_field_pat)
    (match_branch)
    (term)
    (pattern)
    (last_elem_pat)
    (enum)
    (ident)
  ]
  .
  ["," ";"] @append_spaced_scoped_softline
  .
  (comment)? @do_nothing
)

; Enums and records can have a `;` at the very beginning; allow spaces after
; these ones also.
(_
  (#scope_id! "container")
  .
  ";" @append_spaced_scoped_softline
  .
  (comment)? @do_nothing
)
