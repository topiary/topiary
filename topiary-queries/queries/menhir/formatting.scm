; -------------------------------------------------------------------------
; Comments
; -------------------------------------------------------------------------
[
  (comment)
  (ocaml_comment)
] @multi_line_indent_all @allow_blank_line_before @prepend_input_softline

(
  [
    (comment)
    (ocaml_comment)
  ] @append_input_softline
)

; Line comments consume the rest of the line.
(line_comment) @append_hardline @prepend_space @allow_blank_line_before

; -------------------------------------------------------------------------
; Declarations (e.g. %token, %type, %start)
; -------------------------------------------------------------------------
(declaration) @prepend_hardline @allow_blank_line_before

; Preserve optional blank lines around section separators.
(source_file
  "%%" @allow_blank_line_before @prepend_hardline @append_hardline
  .
  (_) @allow_blank_line_before
)

(declaration
  [
    "%token"
    "%type"
    "%start"
    "%inline"
    "%left"
    "%right"
    "%nonassoc"
    "%parameter"
  ] @append_space
)

; -------------------------------------------------------------------------
; Headers (%{ ... %})
; -------------------------------------------------------------------------
(header
  "%{" @append_hardline
  (ocaml) @multi_line_indent_all
  "%}" @prepend_hardline
) @append_hardline @allow_blank_line_before

; -------------------------------------------------------------------------
; Types (< ... >)
; -------------------------------------------------------------------------
(type
  "<"
  (ocaml_type)
  ">" @append_space
)

; -------------------------------------------------------------------------
; Tokens & Terminal Aliases
; -------------------------------------------------------------------------
; This scope spans the complete token declaration. Each terminal below uses a
; scoped softline, so a declaration written on one line keeps spaces between
; terminals, while any multiline declaration breaks and indents every terminal.
(declaration
  (#scope_id! "declaration")
  "%token" @append_indent_start
) @prepend_begin_scope @append_end_scope @append_indent_end

(terminal_alias_attrs
  (#scope_id! "declaration")
) @prepend_spaced_scoped_softline

(terminal_alias_attrs
  (uid) @append_space
)

(non_terminal) @append_spaced_softline
(strict_actual) @append_spaced_softline

; -------------------------------------------------------------------------
; Rules
; -------------------------------------------------------------------------
[
  (old_rule)
  (new_rule)
] @append_hardline @allow_blank_line_before

(old_rule
  "," @append_space
)

(new_rule
  "," @append_space
)

(flags) @append_space

; A single-production rule can contain a multiline action while its production
; still begins beside ":". The regular scope spans the rule, but its measuring
; scope covers only the gap from ":" to the production's first child. That gap,
; rather than the action body, therefore decides whether the rule is multiline.
; The final anchor after production_group excludes rules with later productions.
(old_rule
  (#scope_id! "menhir_old_rule_first_prod")
  ":" @append_input_softline @append_begin_measuring_scope
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  (production_group
    .
    "|"?
    .
    (_) @prepend_end_measuring_scope
  )
  .
) @prepend_begin_scope @append_end_scope

; In classic syntax, a separator between productions is a direct child of
; old_rule; a "|" between producers is nested inside production_group. Matching
; the direct child therefore identifies a multiple-production rule. This scope
; runs from ":" through the entire rule, so it is multiline exactly when the
; productions are split across source lines.
(old_rule
  (#scope_id! "menhir_old_rule_productions")
  ":" @prepend_begin_scope
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  (production_group)
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  "|"
) @append_end_scope

; The first production's scoped softline belongs to the full-rule scope: it is
; a space after ":" in compact layout and a hardline in vertical layout.
(old_rule
  ":"
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  (production_group
    .
    "|"?
    .
    (_) @prepend_spaced_scoped_softline
  )
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  "|"
  (#scope_id! "menhir_old_rule_productions")
)

; These captures share the full-rule scope above. In multiline layout they add
; one indentation level and synthesize the leading "|"; in compact layout the
; scoped predicate disables both directives.
(old_rule
  ":" @append_indent_start
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  (production_group
    .
    "|"?
    .
    (_) @prepend_delimiter @prepend_space
  )
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  "|"
  (#scope_id! "menhir_old_rule_productions")
  (#delimiter! "|")
  (#multi_line_scope_only! "menhir_old_rule_productions")
) @append_indent_end

; The single-production measuring scope likewise controls indentation. Keeping
; these directives separate lets them attach to the first and last rule leaves.
(old_rule
  ":" @append_indent_start
  (#scope_id! "menhir_old_rule_first_prod")
  (#multi_line_scope_only! "menhir_old_rule_first_prod")
)

(old_rule
  (#scope_id! "menhir_old_rule_first_prod")
  (#multi_line_scope_only! "menhir_old_rule_first_prod")
) @append_indent_end

; Normalize an optional leading "|" by deleting the source leaf. Multiline
; layout reinserts it on the production's first child; attaching it there keeps
; the generated delimiter outside the deleted leaf's region.
(old_rule
  ":"
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  (production_group
    .
    "|" @delete
  )
)

(old_rule
  ":"
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  (production_group
    .
    "|"?
    .
    (_) @prepend_delimiter @prepend_space
  )
  (#delimiter! "|")
  (#multi_line_scope_only! "menhir_old_rule_first_prod")
)

(new_rule
  "let" @append_space
)

; This is the modern equivalent of menhir_old_rule_first_prod. The regular scope
; spans new_rule, while the measuring scope covers only the gap from the equality
; symbol to the first expression child. The final expression anchor excludes
; rules with later alternatives.
(new_rule
  (#scope_id! "menhir_new_rule_first_alt")
  (equality_symbol) @prepend_space @append_begin_measuring_scope
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  (expression
    .
    "|"?
    .
    (_) @prepend_end_measuring_scope
    .
  )
) @prepend_begin_scope @append_end_scope

; When that measuring scope detects a source break, open indentation at the
; equality symbol and close it at the rule's final leaf.
(new_rule
  (equality_symbol) @append_indent_start
  (#scope_id! "menhir_new_rule_first_alt")
  (#multi_line_scope_only! "menhir_new_rule_first_alt")
)

(new_rule
  (#scope_id! "menhir_new_rule_first_alt")
  (#multi_line_scope_only! "menhir_new_rule_first_alt")
) @append_indent_end

; The named equality_symbol does not track line_break_after like its leaf token;
; attaching the input softline to expression preserves the same source gap.
(new_rule
  (expression) @prepend_input_softline
)

; A non-leading "|" inside expression identifies multiple modern alternatives.
; This scope begins at the equality symbol and ends with new_rule, so compact
; alternatives yield spaces and source alternatives split over lines yield
; hardlines throughout.
(new_rule
  (#scope_id! "menhir_new_rule_alternatives")
  (equality_symbol) @prepend_space @prepend_begin_scope
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  (expression
    .
    "|"?
    .
    (_)
    .
    "|"
  )
) @append_end_scope

; The first alternative uses the full-rule scope to choose a space after the
; equality symbol for compact layout or a hardline for vertical layout.
(new_rule
  (equality_symbol)
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  (expression
    .
    "|"?
    .
    (_) @prepend_spaced_scoped_softline
    .
    "|"
  )
  (#scope_id! "menhir_new_rule_alternatives")
)

; As in classic syntax, multiline layout adds indentation and a generated
; leading "|", while the compact form receives neither.
(new_rule
  (equality_symbol) @append_indent_start
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  (expression
    .
    "|"?
    .
    (_) @prepend_delimiter @prepend_space
    .
    "|"
  )
  (#scope_id! "menhir_new_rule_alternatives")
  (#delimiter! "|")
  (#multi_line_scope_only! "menhir_new_rule_alternatives")
) @append_indent_end

; Delete the optional source "|" so both layout modes start from one form. The
; appropriate scope reinserts it on the first expression child only for
; multiline layout, outside the deleted leaf's region.
(new_rule
  (equality_symbol)
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  (expression
    .
    "|" @delete
  )
)

(new_rule
  (equality_symbol)
  .
  [(comment) (ocaml_comment) (line_comment)]*
  .
  (expression
    .
    "|"?
    .
    (_) @prepend_delimiter @prepend_space
  )
  (#delimiter! "|")
  (#multi_line_scope_only! "menhir_new_rule_first_alt")
)

; -------------------------------------------------------------------------
; Productions (Alternative Branches)
; -------------------------------------------------------------------------
; These patterns match only non-leading separators. Their scoped softlines use
; the full multiple-production scopes above, making every separator a space in
; compact layout or a hardline in vertical layout.
(old_rule
  "|" @prepend_spaced_scoped_softline @append_space
  (#scope_id! "menhir_old_rule_productions")
)

(production_group
  (_)
  .
  "|" @prepend_input_softline @append_space
)

(expression
  (_)
  .
  "|" @prepend_spaced_scoped_softline @append_space
  (#scope_id! "menhir_new_rule_alternatives")
)

(seq_expression
  "=" @append_space @prepend_space
)

(seq_expression
  ";" @append_space
)

; -------------------------------------------------------------------------
; Producers (Symbols in a Production)
; -------------------------------------------------------------------------
(producer
  (lid) @append_space
  "=" @append_space
)

(producer) @append_space

; Cancel the producer's generated trailing space before a semicolon.
(producer ";" @prepend_antispace)

; -------------------------------------------------------------------------
; Actuals (Rule Arguments)
; -------------------------------------------------------------------------
(actual
  "," @append_space
)

(symbol_expression
  "," @append_space
)

; -------------------------------------------------------------------------
; Semantic Actions ({ ... })
; -------------------------------------------------------------------------
; The inner action scope below sees only the braces and OCaml body. For classic
; syntax, widen a second scope from the preceding producer through the action.
; A source break before the action then makes the brace softlines hardlines,
; moving the break inside the braces instead of leaving "{" on its own line.
(production_group
  (_) @prepend_begin_scope
  .
  (action
    "{" @append_spaced_scoped_softline
    "}" @prepend_spaced_scoped_softline
  ) @append_end_scope
  .
  (#scope_id! "classic_action_after_producers")
)

; Modern actions are nested beneath the sequence following its final semicolon,
; so the equivalent widened scope begins at that semicolon and ends at action.
(seq_expression
  ";" @append_begin_scope
  .
  (seq_expression
    (action_expression
      (menhir_action
        (action
          "{" @append_spaced_scoped_softline
          "}" @prepend_spaced_scoped_softline
        ) @append_end_scope
      )
    )
  )
  (#scope_id! "modern_action_after_semicolon")
)

; This scope decides whether the OCaml body fits inline between spaced brace
; softlines. When multiline, the softlines become hardlines and the body is
; indented one level; multi_line_indent_all shifts every embedded OCaml line.
(action
  (#scope_id! "action")
  "{" @append_indent_start @append_spaced_scoped_softline
  (ocaml) @multi_line_indent_all
  "}" @prepend_indent_end @prepend_spaced_scoped_softline
) @prepend_begin_scope @append_end_scope
