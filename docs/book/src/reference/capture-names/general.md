# General advice

This chapter contains general advice on writing Tree-sitter queries and
also some specific, important Topiary semantics.

## `@leaf`

Some nodes should not have their contents formatted at all; the classic
example being string literals. The `@leaf` capture will mark such nodes
as leaves -- even if they admit their own structure, by virtue of the
grammar -- and leave them unformatted.

### Example

```scheme
; Don't format strings or comments
[
  (string)
  (comment)
] @leaf
```

<div class="warning">

This can make it tricky to format strings that allow interpolation. In
such cases, ideally the grammar would expose this structure, such that
the non-interpolated parts of the string can be `@leaf`.

</div>

## `@do_nothing`

If any of the captures in a query match are `@do_nothing`, then the
entire match will be ignored. This is useful for cancelling a formatting
query based on context.

### Example

```scheme
; Put a semicolon delimiter after field declarations, unless they already have
; one, in which case we do nothing.
(
  (field_declaration) @append_delimiter
  .
  ";"* @do_nothing

  (#delimiter! ";")
)
```

<div class="warning">

Nodes which are annotated with `@do_nothing` ought to be
[quantified][tree-sitter:quantifiers] with Tree-sitter's `*` (zero or
more matches) or `?` (at most one match) operators, to define a pattern
where the exceptional node _could_ appear. Without, the `@do_nothing`
capture will always be applied and the query will be cancelled
regardless.

</div>

## `#query_name!`

When the logging verbosity is set to `-vv` or higher (see [runtime
dialogue](../../cli/dialogue.md#logging)), Topiary outputs information
about which queries are matched, for instance:

```
[2024-10-08T15:48:13Z INFO  topiary_core::tree_sitter] Processing match: LocalQueryMatch { pattern_index: 17, captures: [ {Node "," (1,3) - (1,4)} ] } at location (286,1)
```

Here, `pattern_index: 17` means that the 17th (0-based) pattern in the
query file has matched. Counting patterns in the query file -- not to
mention the potential for off-by-one errors -- is not a great developer
experience!

As such, the optional predicate `#query_name!`, taking a string
argument, can be added to any query. It will modify the log line to
display its argument, to aid debugging.

### Example

Considering the log line above, and let us assume that the query at
`location (286,1)` is:

```scheme
(
  "," @append_space
  .
  (_)
)
```

If we add a `query_name` predicate:

```scheme
(
  "," @append_space
  .
  (_)

  (#query_name! "comma spacing")
)
```

Then the log line will become:

```
[2024-10-08T15:48:13Z INFO  topiary_core::tree_sitter] Processing match of query "comma spacing": LocalQueryMatch { pattern_index: 17, captures: [ {Node "," (1,3) - (1,4)} ] } at location (286,1)
```

## Tree-sitter predicates

Tree-sitter supports a number of predicates by default, which allow for
fine-tuning queries. These are discussed in the [Tree-sitter
documentation][tree-sitter:predicates] and outlined here:

- **`#eq?`:** Checks a direct match against a capture or string.
- **`#match?`:** Checks a match against a regular expression.
- **`#any-of?`:** Checks a match against a list of strings.
- Prefixing **`not-`** negates any of the above predicates.

> **Note**\
> Topiary does not allow arbitrary capture names; just those it defines
> for formatting. The Tree-sitter predicates expect a capture name and,
> as such, this can make using them with Topiary a little unwieldy (see
> [issue #824][topiary:#824]).

<div class="warning">
Topiary uses the Rust implementation of Tree-sitter which may lag behind
the reference C implementation. The above predicates have been confirmed
to work, but others mentioned in the Tree-sitter documentation may not.

For example, as of writing, while the documented `any-` prefix for `eq`
and `match` is recognised by Topiary's Tree-sitter, it doesn't appear to
work as advertised.
</div>

## Query and capture precedence

Formatting is not necessarily invariant over the order of queries. For
example, queries that add delimiters or remove nodes can have a
different effect on the formatted output depending on the order in which
they appear in the query file.

Consider, say, the following two queries for the Bash grammar:

```scheme
; Query A: Append semicolon
(
  (word) @append_delimiter
  .
  ";"? @do_nothing

  (#delimiter! ";")
)

; Query B: Surround with quotes
(
  "\""? @do_nothing
  .
  (word) @prepend_delimiter @append_delimiter
  .
  "\""? @do_nothing

  (#delimiter! "\"")
)
```

In the order presented above (`A`, then `B`), then the input `foo` will
be formatted as:

```
"foo;"
```

In the opposite order (`B`, then `A`), Topiary will however produce the
following output:

```
"foo";
```

A similar consideration exists for capture names. That is, while most
captures do not meaningfully affect one another, there are three notable
exceptions:

1. `@do_nothing` (see [above](#do_nothing)) will cancel all other
   captures in a matched query. This takes the highest priority.

2. `@delete` (see [insertion and deletion](insertion-and-deletion.md#delete))
   will delete any matched node, providing the matching query is not
   cancelled.

3. `@leaf` (see [above](#leaf)) will suppress formatting within that
   node, even if it admits some internal structure. However, leaf nodes
   are still subject to deletion.

> **Note**\
> While not in the same league as the above, also note that antispaces
> will cancel out all inserted spaces (see [horizontal
> spacing](horizontal-spacing.md)).

## Captures are always postfix

Note that a capture is put after the node it is associated with. If you
want to put a space in front of a node, for example, you do so like
this:

```scheme
(infix_operator) @prepend_space
```

This, on the other hand, will not work:

```scheme
@append_space (infix_operator)
```

## A note on anchors

The behaviour of ["anchors"][tree-sitter:anchors] can be
counter-intuitive. Consider, for instance, the following query:

```scheme
(
  (list_entry) @append_space
  .
)
```

One might assume that this query only matches the final element in the
list but this is not true. Since we did not explicitly match a parent
node, the engine will match on every `list_entry`. After all, when
looking only at the nodes in the query, the `list_entry` is indeed the
last node.

To resolve this issue, match explicitly on the parent node:

```scheme
(list
  (list_entry) @append_space
  .
)
```

Or even implicitly:

```scheme
(_
  (list_entry) @append_space
  .
)
```

<div class="warning">

Note that while anchors can be defined between anonymous nodes, if they
are given as explicit terminals, anonymous nodes that _interpose_ an
anchor's terminals (named or anonymous) will be skipped over.

For example, in this Bash code:

```bash
if this; then that; fi
```

The following query matches the nodes indicated in the comments:

```scheme
(if_statement
  (_)  ; will match "this"
  .
  (_)  ; will match "that"
)
```

In the Bash grammar, `this` and `that` are named nodes, but are
interposed by the `;` and `then` anonymous nodes, which are ignored by
the anchor.

</div>

<div class="warning">

Using anchors wherever possible is highly recommended, otherwise queries
can become too general and over-match, despite resulting in the same
outcome. This can significantly impact formatting performance.

For example, imagine the list `[1 2 3 4 5]`. Adding spaces between
elements would be best expressed as:

```scheme
(list
  (element) @append_space
  .
  (element)
)
```

Here, this query will match 4 times -- `1 2`, `2 3`, `3 4` and `4 5` --
and Topiary will insert exactly the right number of spaces.

If we remove the anchor, it will match 10 times -- `1 2`, `1 3`, `1 4`,
`1 5`, `2 3`, `2 4`, `2 5`, `3 4`, `3 5` and `4 5` -- so Topiary does
more than twice as much work, only for [subsequent processing](../formatting-pipeline.md#atom-processing)
to remove all those extraneous spaces.

</div>

<!-- Links -->
[topiary:#824]: https://github.com/topiary/topiary/issues/824
[tree-sitter:anchors]: https://tree-sitter.github.io/tree-sitter/using-parsers/queries/2-operators.html#anchors
[tree-sitter:predicates]: https://tree-sitter.github.io/tree-sitter/using-parsers/queries/3-predicates-and-directives.html
[tree-sitter:quantifiers]: https://tree-sitter.github.io/tree-sitter/using-parsers/queries/2-operators.html#quantification-operators
