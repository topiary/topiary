# Indentation

Indentation is a special form of horizontal spacing, where "blocks" are
defined using start and end markers. Each block, which can be nested,
sets an increasing indentation level which, when formatted in the
output, will be prefixed with an appropriate number of indentation
strings (defined in the [language configuration](../../cli/configuration.md#indentation)).

## `@append_indent_start` / `@prepend_indent_start`

The matched nodes will trigger indentation before (or, respectively,
after) them. This will only apply to the lines following, until an
indentation end is signalled. If indentation is started and ended on the
same line, nothing will happen. This is useful, because we get the
correct behaviour whether a node is formatted as single-line or
multi-line. It is important that all indentation starts and ends are
balanced.

> **Note**\
> If indentation is not balanced, the formatting will (probably) not
> fail, but a warning will be logged.

### Example

```scheme
; Start an indented block after these
[
  "begin"
  "else"
  "then"
  "{"
] @append_indent_start
```

## `@append_indent_end` / `@prepend_indent_end`

The matched nodes will trigger that indentation ends before (or,
respectively, after) them.

### Example

```scheme
; End the indented block before these
[
  "end"
  "}"
] @prepend_indent_end

; End the indented block after these
[
  (else_clause)
  (then_clause)
] @append_indent_end
```

## `@multi_line_string`

to do

## `@multi_line_indent_all`

To be used on comments, or other leaf nodes, to indicate that we should
indent all its lines, not just the first.

### Example

```scheme
(comment) @multi_line_indent_all
```

## `@single_line_no_indent`

The matched node will be printed alone, on a single line, regardless of
any indentation level. That is, this capture temporarily suspends normal
indentation for the node that is matched.

### Example

```scheme
; line number directives must be alone on their line, and can't be indented
(line_number_directive) @single_line_no_indent
```
