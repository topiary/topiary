;; -----------------------------------------------------------------------------
;; Comments
;; -----------------------------------------------------------------------------
(line_comment) @append_hardline @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Modules
;; -----------------------------------------------------------------------------
(module
  "module" @append_space
  name: (identifier) @append_space
  base: (iri)? @append_space
  "version"? @append_space
  version_info: (quoted_string)? @append_space
  version_uri: (iri)? @append_space
)

(module_body) @prepend_space

;; -----------------------------------------------------------------------------
;; Indentation
;; -----------------------------------------------------------------------------
["is" "of"] @append_indent_start @append_hardline
["end"] @allow_blank_line_before @prepend_indent_end

;; -----------------------------------------------------------------------------
;; Imports
;; -----------------------------------------------------------------------------
(import_statement
  "import" @append_space
) @allow_blank_line_before @append_hardline

(import_statement
  "[" @append_indent_start @append_spaced_softline
  "]" @prepend_indent_end
)

[(member_import) (module_import)] @append_spaced_softline

(member_import "as" @prepend_space @append_space)
(module_import "as" @prepend_space @append_space)

;; -----------------------------------------------------------------------------
;; Annotations
;; -----------------------------------------------------------------------------
(annotation) @allow_blank_line_before @append_hardline

;; -----------------------------------------------------------------------------
;; Annotations >> Properties
;; -----------------------------------------------------------------------------
(annotation_property "=" @prepend_space @append_space) @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Annotations >> Constraints
;; -----------------------------------------------------------------------------
(constraint "assert" @append_space name: (identifier) @append_space)

;; -----------------------------------------------------------------------------
;; Annotations >> Constraints >> Informal
;; -----------------------------------------------------------------------------
(informal_constraint "=" @prepend_space @append_space)

;; -----------------------------------------------------------------------------
;; Annotations >> Constraints >> Formal
;; -----------------------------------------------------------------------------
(formal_constraint "end" @prepend_hardline)

(constraint_environment
  "with" @append_indent_start @append_hardline
) @append_indent_end

(equation "=" @prepend_space @append_space)

(inequation
  [
    (op_inequality)
    (op_less_than)
    (op_greater_than)
    (op_less_than_or_equal)
    (op_greater_than_or_equal)
  ] @prepend_space @append_space
)

(binary_boolean_sentence
  [
    (logical_op_conjunction)
    (logical_op_disjunction)
    (logical_op_exclusive_disjunction)
    (logical_op_implication)
    (logical_op_biconditional)
  ] @prepend_space @append_space
)

(logical_op_negation) @append_space

(quantified_sentence "," @prepend_antispace @append_space)

(quantified_variable_binding
  [
    (logical_quantifier_universal)
    (logical_quantifier_existential)
  ] @append_space
)

(quantified_variable (set_op_membership) @prepend_space @append_space)

(actual_arguments
  [
    ((term) . ")" @do_nothing)
    ((term) . (term) @prepend_space)
  ]
)

(function_def
) @allow_blank_line_before @append_hardline @prepend_spaced_softline
(function_signature
  "def" @append_space
  name: (identifier) @append_space
)

(function_signature
  "(" @append_indent_start
  [
    ((function_parameter) . ")" @do_nothing)
    ((function_parameter) . (function_parameter) @prepend_space)
  ]
  ")" @prepend_indent_end
)

(function_signature
  ["->" "→"] @prepend_space @append_space
  (function_type_reference) @append_space
)

(function_parameter
  name: (identifier) @append_space
  ["->" "→"] @append_space
)
(function_body
  (function_op_by_definition) @prepend_space @append_space
) @prepend_hardline @prepend_indent_start @append_indent_end

(function_cardinality_expression
  "{" @append_antispace
  ordering: (_)? @append_space
  uniqueness: (_)? @append_space
  "}" @prepend_antispace
) @append_space

(sequence_builder
  "{" @append_spaced_softline @append_indent_start
  (set_op_builder) @prepend_spaced_softline @prepend_indent_end @append_indent_start
  "}" @prepend_indent_end @prepend_spaced_softline
)

(sequence_builder (variable) @append_space)

(sequence_builder_body) @prepend_spaced_softline

;; -----------------------------------------------------------------------------
;; Definitions
;; -----------------------------------------------------------------------------
(definition) @append_hardline @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Definitions >> Class
;; -----------------------------------------------------------------------------
(type_class_def
  "class" @append_space
  name: (identifier) @append_space
) @allow_blank_line_before

(method_def
) @allow_blank_line_before @append_hardline

(method_def
  (function_body)
  .
  (annotation_only_body) @prepend_space
)

;; -----------------------------------------------------------------------------
;; Definitions >> Datatype
;; -----------------------------------------------------------------------------
(data_type_def
  "datatype" @append_space
  name: (identifier) @append_space
  ["<-" "←"] @append_space
  opaque: (opaque)? @append_space
  base: (_) @append_space
) @allow_blank_line_before

;;(data_type_boolean_base
;; (boolean_connective) @append_space
;; "[" @append_space
;; first: (_) @append_space
;; rest: (_) @append_space
;; "]" @append_space)

(datatype_def_restriction
  "{" @append_indent_start @append_hardline
  "}" @prepend_indent_end
)

(length_restriction_facet "=" @prepend_space @append_space)
(digit_restriction_facet "=" @prepend_space @append_space)
(value_restriction_facet "=" @prepend_space @append_space)
(tz_restriction_facet "=" @prepend_space @append_space)
(pattern_restriction_facet "=" @prepend_space @append_space)
(kw_is_fixed) @append_space

(pattern_restriction_facet
  "[" @append_indent_start @append_spaced_softline
  (quoted_string)* @append_spaced_softline
  "]" @prepend_indent_end
)

(datatype_def_restriction
  [
    (length_restriction_facet)
    (digit_restriction_facet)
    (value_restriction_facet)
    (tz_restriction_facet)
    (pattern_restriction_facet)
  ] @append_hardline
)

(data_type_def
  (datatype_def_restriction)
  .
  (annotation_only_body) @prepend_space
)

;; -----------------------------------------------------------------------------
;; Definitions >> Dimension
;; -----------------------------------------------------------------------------
(dimension_def
  "dimension" @append_space
  name: (identifier) @append_space
)

(source_entity
  "source" @append_space
  entity: (identifier_reference) @append_space
  (
    "with" @append_space
    (
      "[" @append_indent_start @append_spaced_softline
      (identifier)* @append_spaced_softline
      "]" @prepend_indent_end
    )?
  )?
) @append_hardline @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Definitions >> Entity
;; -----------------------------------------------------------------------------
(entity_def
  "entity" @append_space
  name: (identifier) @append_space
)

;; -----------------------------------------------------------------------------
;; Definitions >> Enum
;; -----------------------------------------------------------------------------
(enum_def
  "enum" @append_space
  name: (identifier) @append_space
)

;; -----------------------------------------------------------------------------
;; Definitions >> Event
;; -----------------------------------------------------------------------------
(event_def
  "event" @append_space
  name: (identifier) @append_space
)

;; -----------------------------------------------------------------------------
;; Definitions >> Property [[ nothing required, uses member_def rules ]]
;; -----------------------------------------------------------------------------
(property_def
  "property" @append_space
)

;; -----------------------------------------------------------------------------
;; Definitions >> RDF
;; -----------------------------------------------------------------------------
(rdf_def
  "rdf" @append_space
  name: (identifier) @append_space
)

(rdf_types
  "type" @append_space
  [
    (
      "[" @append_indent_start @append_spaced_softline
      (identifier_reference)* @append_spaced_softline
      "]" @prepend_indent_end @append_space
    )
    (identifier_reference) @append_space
  ]
)

;; -----------------------------------------------------------------------------
;; Definitions >> Structure
;; -----------------------------------------------------------------------------
(structure_def
  "structure" @append_space
  name: (identifier) @append_space
)

;; -----------------------------------------------------------------------------
;; Definitions >> Union
;; -----------------------------------------------------------------------------
(union_def
  "union" @append_space
  name: (identifier) @append_space
)

;; -----------------------------------------------------------------------------
;; Members
;; -----------------------------------------------------------------------------
(entity_identity "identity" @append_space)

(member) @append_hardline @allow_blank_line_before

(member_def
  name: (identifier) @append_space
  ["->" "→"] @append_space
  cardinality: (cardinality_expression)? @append_space
  target: (type_reference) @append_space
)

(cardinality_expression
  "{" @append_antispace
  ordering: (_)? @append_space
  uniqueness: (_)? @append_space
  "}" @prepend_antispace
)

(property_ref "ref" @append_space)

;; -----------------------------------------------------------------------------
;; Variants
;; -----------------------------------------------------------------------------
(value_variant
  body: (annotation_only_body)? @prepend_space
) @append_hardline @allow_blank_line_before

(type_variant
  "as"? @prepend_space
  rename: (identifier)? @prepend_space
  body: (annotation_only_body)? @prepend_space
) @append_hardline @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Values
;; -----------------------------------------------------------------------------
[
  (boolean)
  (integer)
  (unsigned)
  ;;(rational)
  (decimal)
  (double)
  (string)
  (iri)
] @leaf

(binary "#[" @append_space (hex_byte)* @append_space)

(sequence_of_values
  (
    (sequence_ordering)? @append_space
    .
    (sequence_uniqueness)
    "}" @append_space
  )?
  "[" @append_indent_start @append_spaced_softline
  element: (_)* @append_spaced_softline
  "]" @prepend_indent_end
)

(sequence_of_values
)

(sequence_of_predicate_values
  (
    (sequence_ordering)? @append_space
    .
    (sequence_uniqueness)
    "}" @append_space
  )?
  (
    "[" @append_indent_start @append_spaced_softline
    element: (_)* @append_spaced_softline
    "]" @prepend_indent_end
  )
)

(value_constructor
  "(" @append_antispace
  ")" @prepend_antispace
)

(mapping_value
  ["->" "→"] @prepend_space @append_space
)

;; -----------------------------------------------------------------------------
;; Misc
;; -----------------------------------------------------------------------------

(mapping_type
  ["->" "→"] @prepend_space @append_space
)
