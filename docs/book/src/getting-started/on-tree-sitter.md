# Tree-sitter and its queries

From the [Tree-sitter documentation][tree-sitter:intro]:

> Tree-sitter is a parser generator tool and an incremental parsing
> library. It can build a concrete syntax tree for a source file and
> efficiently update the syntax tree as the source file is edited.

Any non-trivial formatter needs to work with the code's syntax tree.
Topiary uses Tree-sitter to this end, which precludes the need to write
a new parser each time formatting for a new language is needed.

A number of [Tree-sitter grammars][tree-sitter:known-grammars] already
exist and these languages are, therefore, potential targets for Topiary.
Even in the case when a Tree-sitter grammar doesn't exist for some
target language, writing one is significantly easier than writing a
parser from scratch and benefits from other tooling in the Tree-sitter
ecosystem.

## Tree-sitter queries

Tree-sitter exposes a query interface, which allows you to express
syntactic patterns in a simple DSL that Tree-sitter will match against
your source code. You can think of it a little bit like "regular
expressions for syntax trees". While regular expressions match linear
strings, and thus look like a bit like a string, Tree-sitter queries
match against syntax trees, so they look a bit like a tree. They are
expressed using S-expressions, similar to Lisp, using the node names
that are defined in the grammar.

For example, say you wanted to find `for` loops with an `if` statement
as the first child. The query for this might look something like:

```scheme
(for_loop
  .
  (if_statement)
)
```

The syntax for Tree-sitter queries is described in the [Tree-sitter
documentation][tree-sitter:queries],[^support] which we won't go into
here. However, there is one very important concept in Tree-sitter
queries, as far as Topiary is concerned: what Tree-sitter calls "capture
names".

When you write a query and you want to capture a specific node in that
query, rather than the entire subtree, you can annotate the node with a
"capture name". These are represented with an `@` character, followed by
an identifier. (To stretch our regular expression analogy, these would
be similar to named groups.)

In our above example, say we actually only care about `if` statements
that appear as the first child of `for` loops; we don't care about the
rest of the subtree. Then we could add a capture name as follows:

```scheme
(for_loop
  .
  (if_statement) @my_important_node
)
```

It's this mechanism that Topiary relies on to perform formatting. That
is, queries are used to identify syntactic constructions of interest,
then specific capture names -- for which Topiary [defines particular
semantics](../reference/capture-names/index.md) -- apply respective
formatting to those nodes within that and every matching subtree.

Say, for example, we always want an `if` statement that appears as the
first child of a `for` loop to be on a new line. For this, we could use
the [`@prepend_hardline` capture name](../reference/capture-names/vertical-spacing.md#append_hardline--prepend_hardline):

```scheme
(for_loop
  .
  (if_statement) @prepend_hardline
)
```

This is the [essence of Topiary](../reference/formatting-pipeline.md#query-matching).

<div class="warning">
This example is contrived for illustrative purposes. The queries you
would actually write would probably be much more general, so you don't
have to enumerate every syntactic possibility in which a line break
needs to be inserted. However, as Tree-sitter is so flexible, if
exceptional rules like this exist, they can easily be accommodated.
</div>

### Topiary language query files

A language query file, usually given the `.scm` extension, is a
collection of such queries with appropriate formatting capture names.
Taken in aggregate, when applied to source code files by Topiary, they
define a formatting style for the language in question.

<div class="warning">

Topiary makes no assumption about a language's token separators. When it
parses any input source into a collection of nodes, it will only apply
formatting that has been _explicitly defined_ with capture names, in a
query file. This can result in any unspecified nodes losing their token
separators (e.g., spacing) after formatting. That is, nodes can be
"squashed" together, which can change (or even break) the code's
semantics.

For example, with the Bash Tree-sitter grammar, the command invocation
`command arg1 arg2` has a syntax tree that looks like this:

```
command
  command_name
    word
  word
  word
```

A naive formatting query might just put a space after the command
itself, in which case the arguments would coalesce. An improvement on
this would be to account for the `(word)` arguments, but then that
becomes tightly coupled to this use-case (e.g., `(string)` nodes can
also be command arguments). A better insight is to observe that pairs of
nodes under the `(command)` node -- whatever they may be -- should have
a space between them:

```scm
; Naive query (incorrect; neglects arguments)
; Results in: command arg1arg2
(command_name) @append_space


; Improved queries (incorrect; too specific)
; Results in: command arg1 arg2
; Failure mode if the first argument is a string: command "arg1"arg2
(command
  (command_name) @append_space
)

(command
  (word) @append_space
)


; Correct query (if we ignore everything besides commands)
; Results in: command arg1 arg2
; Handles string arguments: command "arg1" arg2
(command
  (_) @append_space
  .
  (_)
)
```

This can seem counter-intuitive, or even frustrating, to new-comers.
However, it stands to reason and nudges the creation of suitably general
rules to correct the spacing.

</div>

<!-- Footnotes -->

[^support]:
    Note that Topiary may not support the entirety of the Tree-sitter
    query syntax, as it uses the Rust implementation of Tree-sitter,
    which may lag behind the reference C implementation.

<!-- Links -->
[tree-sitter:intro]: https://tree-sitter.github.io/tree-sitter
[tree-sitter:known-grammars]: https://github.com/tree-sitter/tree-sitter/wiki/List-of-parsers
[tree-sitter:queries]: https://tree-sitter.github.io/tree-sitter/using-parsers/queries/index.html
