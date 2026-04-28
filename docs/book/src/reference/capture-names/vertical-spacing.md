# Vertical spacing

## Hardlines and softlines

Topiary makes a distinction between "hardlines" and "softlines".
Hardlines are inserted regardless of the context; whereas softlines are
more like points where line breaks can be inserted, based on context.

In particular, Topiary goes through the CST nodes and detects all that
span more than one line. This is interpreted as an indication from the
programmer who wrote the input that the node in question should be
formatted as multi-line; while any other nodes will be formatted as
single-line.

Whenever a query match inserts a softline, the softline will be expanded
to a line break if the **immediate parent of the node marked with the
softline capture** spans more than one line. Another way to look at this
is that these capture names are a convenience over their
[scoped](scopes.md) equivalent, with the scope implicitly set to the
parent node.

See:

- [`@append_hardline` / `@prepend_hardline`](#append_hardline--prepend_hardline)
- [`@append_empty_softline` / `@prepend_empty_softline`](#append_empty_softline--prepend_empty_softline)
- [`@append_spaced_softline` / `@prepend_spaced_softline`](#append_spaced_softline--prepend_spaced_softline)
- [`@append_input_softline` / `@prepend_input_softline`](#append_input_softline--prepend_input_softline)
- [`@append_empty_input_softline` / `@prepend_empty_input_softline`](#append_empty_input_softline--prepend_empty_input_softline)

<div class="warning">

Topiary inserts `\n` as a line break on all platforms (including
Windows).

</div>

> **Note**\
> The single- and multi-lined context can also be used to drive
> arbitrary queries, not necessarily related to vertical spacing; see
> [below](#testing-context-with-predicates).

### Understanding the different line break captures

| Type                 | Single-Line Context | Multi-Line Context |
| :------------------- | :------------------ | :----------------- |
| Hardline             | Line break          | Line break         |
| Empty Softline       | Nothing             | Line break         |
| Spaced Softline      | Space               | Line break         |
| Input Softline       | Space               | Input-Dependent    |
| Empty Input Softline | Nothing             | Input-Dependent    |

"Input softlines" are rendered as line breaks whenever the targeted node
follows/precedes (for append/prepend, respectively) a line break in the
input. Otherwise, they are rendered as nothing or spaces, depending on the capture name.

#### Example

Consider the following JSON, which has been hand-formatted to exhibit
every context under which the different line break capture names
operate:

```json
{
  "single-line": [1, 2, 3, 4],
  "multi-line": [
    1, 2,
    3
    , 4
  ]
}
```

We'll apply a simplified set of JSON formatting queries that:

1. Opens (and closes) an indented block for objects;
2. Each key-value pair gets its own line, with the value split onto a
   second;
3. Applies the different line break capture name on array delimiters.

That is, iterating over each `@LINEBREAK` type, we apply the following:

```scheme
(object . "{" @append_hardline @append_indent_start)
(object "}" @prepend_hardline @prepend_indent_end .)
(object (pair) @prepend_hardline)
(pair . _ ":" @append_hardline)

(array "," @LINEBREAK)
```

The first four formatting queries are just for clarity's sake. The last
query is what's important; the results of which are demonstrated below:

##### `@append_hardline`

```json
{
  "single-line":
  [1,
  2,
  3,
  4],
  "multi-line":
  [1,
  2,
  3,
  4]
}
```

A line break is added after each delimiter.

##### `@prepend_hardline`

```json
{
  "single-line":
  [1
  ,2
  ,3
  ,4],
  "multi-line":
  [1
  ,2
  ,3
  ,4]
}
```

A line break is added before each delimiter.

##### `@append_empty_softline`

```json
{
  "single-line":
  [1,2,3,4],
  "multi-line":
  [1,
  2,
  3,
  4]
}
```

In the single-line context, all whitespace is consumed around
delimiters; in the multi-line context, a line break is added after each
delimiter.

##### `@prepend_empty_softline`

```json
{
  "single-line":
  [1,2,3,4],
  "multi-line":
  [1
  ,2
  ,3
  ,4]
}
```

In the single-line context, all whitespace is consumed around
delimiters; in the multi-line context, a line break is added before each
delimiter.

##### `@append_spaced_softline`

```json
{
  "single-line":
  [1, 2, 3, 4],
  "multi-line":
  [1,
  2,
  3,
  4]
}
```

In the single-line context, a space is added after each delimiter; in
the multi-line context, a line break is added after each delimiter.

##### `@prepend_spaced_softline`

```json
{
  "single-line":
  [1 ,2 ,3 ,4],
  "multi-line":
  [1
  ,2
  ,3
  ,4]
}
```

In the single-line context, a space is added before each delimiter; in
the multi-line context, a line break is added before each delimiter.

##### `@append_input_softline`

```json
{
  "single-line":
  [1, 2, 3, 4],
  "multi-line":
  [1, 2,
  3, 4]
}
```

In the single-line context, a space is added after each delimiter; in
the multi-line context, a line break is added after the delimiters that
had a line break after them in the original input.

##### `@prepend_input_softline`

```json
{
  "single-line":
  [1 ,2 ,3 ,4],
  "multi-line":
  [1 ,2 ,3
  ,4]
}
```

In the single-line context, a space is added before each delimiter; in
the multi-line context, a line break is added before the delimiters that
had a line break before them in the original input.

##### `@append_empty_input_softline`

```json
{
  "single-line":
  [1,2,3,4],
  "multi-line":
  [1,2,
  3,4]
}
```

The same as `@append_input_softline` except it does not insert a space.

##### `@prepend_empty_input_softline`

```json
{
  "single-line":
  [1,2,3,4],
  "multi-line":
  [1,2,3
  ,4]
}
```

The same as `@prepend_input_softline` except it does not insert a space.

### Testing context with predicates

Sometimes, similarly to what happens with softlines, we want a query to
match only if the context is single-line, or multi-line. Topiary has
several predicates that achieve this result.

#### `#single_line_only!` / `#multi_line_only!`

These predicates allow the query to trigger only if the matched nodes
are in a single-line (or, respectively, multi-line) context.

> **Note**\
> There are scoped equivalents to these predicates; see
> [scopes](scopes.md#testing-context-with-predicates) for details.

##### Example

```scheme
; Allow (and enforce) the optional "|" before the first match case
; in OCaml if and only if the context is multi-line
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
  (#delimiter! "| ")
  (#multi_line_only!)
)
```

## `@allow_blank_line_before`

The matched nodes will be allowed to have a blank line before them, if
specified in the input. For any other nodes, blank lines will be
removed.

### Example

```scheme
; Allow comments and type definitions to have a blank line above them
[
  (comment)
  (type_definition)
] @allow_blank_line_before
```

## `@append_hardline` / `@prepend_hardline`

The matched nodes will have a line break appended (or, respectively,
prepended) to them.

> **Note**\
> If you wish to insert empty lines -- that is, two line breaks --
> between nodes, this can be emulated with [`@append_delimiter` /
> `@prepend_delimiter`](insertion-and-deletion.md#append_delimiter--prepend_delimiter).
> For example:
>
> ```scheme
> (
>   (block) @append_delimiter
>   .
>   (_)
>
>   (#delimiter! "\n\n")
> )
> ```
>
> However, bear in mind that Topiary's normal [post-processing](../formatting-pipeline.md#atom-processing)
> that squashes runs of whitespace will not apply, so queries must be
> written with care to avoid extra, unintended line breaks.

### Example

```scheme
; Consecutive definitions must be separated by line breaks
(
  (value_definition) @append_hardline
  .
  (value_definition)
)
```

## `@append_empty_softline` / `@prepend_empty_softline`

The matched nodes will have an empty softline appended (or,
respectively, prepended) to them. This will be expanded to a line break
for multi-line nodes and to nothing for single-line nodes.

### Example

```scheme
; Put an empty softline before dots, so that in multi-line constructs we start
; new lines for each dot.
(_
  "." @prepend_empty_softline
)
```

## `@append_spaced_softline` / `@prepend_spaced_softline`

The matched nodes will have a spaced softline appended (or,
respectively, prepended) to them. This will be expanded to a line break
for multi-line nodes and to a space for single-line nodes.

### Example

```scheme
; Append spaced softlines, unless there is a comment following.
(
  [
    "begin"
    "else"
    "then"
    "->"
    "{"
    ";"
  ] @append_spaced_softline
  .
  (comment)* @do_nothing
)
```

## `@append_input_softline` / `@prepend_input_softline`

The matched nodes will have an input softline appended (or,
respectively, prepended) to them. An input softline is a line break if
the node has a line break after (or, respectively, before) it in the
input document, otherwise it is a space.

### Example

```scheme
; Input softlines before and after all comments. This means that the input
; decides if a comment should have line breaks before or after. But don't put a
; softline directly in front of commas or semicolons.

(comment) @prepend_input_softline

(
  (comment) @append_input_softline
  .
  [ "," ";" ]* @do_nothing
)
```

## `@append_empty_input_softline` / `@prepend_empty_input_softline`

This option is the empty counterpart to [`@append_input_softline` / `@prepend_input_softline`](#append_input_softline--prepend_input_softline)

This option has the same relation to [`@append_input_softline` / `@prepend_input_softline`](#append_input_softline--prepend_input_softline)
as [`@append_empty_softline` / `@prepend_empty_softline`](#append_empty_softline--prepend_empty_softline)
has to [`@append_spaced_softline` / `@prepend_spaced_softline`](#append_spaced_softline--prepend_spaced_softline)

## `@keep_whitespace`

To be used on leaf nodes. The matched node will keep its trailing `\n` characters.

### Example

```scheme
; keep trailing newlines
(raw_text) @keep_whitespace
```
