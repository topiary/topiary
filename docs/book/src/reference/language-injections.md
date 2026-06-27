# Language injections

Language injections let a host language delegate part of its input to
another Topiary language. This is useful for languages that embed code
from another language in a syntactically distinct region. For example,
OCamllex embeds OCaml code in `(ocaml)` nodes, and Topiary can format
that inner OCaml with the OCaml formatter while the OCamllex formatter
continues to control the surrounding lexer syntax.

Injection declarations live in an `injections.scm` file next to a
language's `formatting.scm` file:

```text
topiary-queries/queries/<language>/formatting.scm
topiary-queries/queries/<language>/injections.scm
```

The formatting query still describes host-language layout. The
injection query only describes which host syntax nodes should be
formatted by another language.

## Query schema

The initial injection schema supports a captured content node and a
static injected language name:

```scheme
(
  (ocaml) @injection.content
  (#injection_language! "ocaml")
)
```

The `@injection.content` capture marks the host node whose source text
will be formatted by the injected language. The
`#injection_language!` predicate declares the Topiary language
identifier to use for that captured content.

Patterns without `#injection_language!` are skipped with a warning.
Patterns without `@injection.content` do not inject anything.

Dynamic language selection is also supported. For example, taking the language name from a Markdown code fence can be done using the `@injection.language` capture:

```scheme
(fenced_code_block
  (info_string
    (language) @injection.language
  )
  (code_fence_content) @injection.content
)
```

Topiary will check for `#injection_language!` first, and if absent, it will fall back to using the text of the `@injection.language` captured node.

## Formatting model

Topiary parses the host input once. If the host language has an
`injections.scm` file, Topiary runs that query against the host tree and
collects the captured spans. Those captured nodes are then treated as
host leaves while the normal host formatting query is applied.

After host atomisation, each injected span is formatted independently
with its resolved inner language. The resulting text replaces the
corresponding host leaf before normal atom post-processing and pretty
printing continue.

Inner formatters produce text from column zero. The host renderer is
still responsible for applying indentation at the point where the
injected leaf is rendered. This works well when the host grammar has a
natural boundary around the injected code, such as OCamllex action
blocks.

There is no host reparse after injected text is rewritten.

## Failure behaviour

Injected formatting is largely robust. If an injection query matches,
Topiary will attempt to resolve the injected language and format the captured span.
If the injected language cannot be resolved (for instance, if the language is not configured or unsupported), Topiary will log a warning and gracefully skip formatting that specific injected span, leaving the original text unchanged. However, if the language *is* resolved but the captured span cannot be successfully formatted due to syntax errors and `tolerate_parsing_errors` is false, formatting of the file may fail.

Idempotence is still checked at the outer formatting level by default.
Injected spans are formatted again during that second pass, so unstable
injected formatting can still make the top-level idempotence check fail.

## CLI behaviour

The CLI resolves injected language names through the normal language
configuration and query discovery paths. Compiled language definitions
are cached by the existing language definition cache, so formatting many
spans of the same injected language does not recompile the grammar or
reload the same queries for every span.

If the CLI cannot resolve the injected language at runtime, formatting
fails. This can happen when a language is missing from the runtime
configuration or its query files cannot be found.

## Limitations

Injected spans are formatted independently. Current injection formatting
cannot express layout decisions that need to measure or choose between
softline layouts across the host/injected boundary.

The model is also stateless. The injected language is determined by the
injection query match itself, not by earlier host-language context. For
example, a host syntax where one declaration changes the language of a
later heredoc is outside the current model.

The injection query should not produce overlapping or nested
`@injection.content` captures. If multiple patterns match the same
region of code, Topiary will attempt to format each one. However,
because an injected span is treated as a forced leaf in the host
language, any nested captures will fail to find their corresponding leaf
node during the rewrite phase, resulting in a formatting error. Query
authors should ensure that injection patterns are mutually exclusive for
any given span of text.
