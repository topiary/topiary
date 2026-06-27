# The formatting pipeline

## Query matching

As discussed in [Tree-sitter and its queries](../getting-started/on-tree-sitter.md),
the essence of Topiary is to apply the formatting queries against an
input. That is, each Tree-sitter query will be applied to the input's
syntax tree, taking into account the relevant capture names against all
matching subtrees. The result of this is effectively serialised into a
list of atomic units -- formatting directives, defined by the matching
capture names, and leaf node content (see also [`@leaf`](capture-names/general.md#leaf))
-- along with the necessary metadata to drive the process.

If the host language defines [language injections](language-injections.md),
Topiary first discovers the injected spans and treats those host nodes as
forced leaves during query matching. The captured leaf content is
rewritten with the inner language's formatted output before atom
post-processing and pretty printing continue.

## Atom processing

The list of atoms from the first step are then processed into a
cleaned-up, canonical form. Specifically:

- Processes [scopes](capture-names/scopes.md);
- Processes [deletions](capture-names/insertion-and-deletion.md#delete);
- Processes [case modifications](capture-names/modification.md#lower_case--upper_case);
- Merges runs of whitespace into a single atom -- or removes them
  entirely, in the presence of an [antispace](capture-names/horizontal-spacing.md#append_antispace--prepend_antispace)
  -- and sorts some remaining, adjacent atoms (e.g., hardlines always
  before spaces, etc.).

> **Note**\
> In the code, this step is referred to as "post-processing"; as in
> "post-query matching-processing", rather than a final step.

## Pretty printing

The pretty printer goes through the processed atom collection and
renders each atom into a stream of text output. For example, an
"indentation start" atom will increase the indent level; that is, all
atoms immediately following a hardline will now be prefixed with the
appropriate indent string, until the respective "indentation end" atom
is reached.

Injected leaves are rendered like any other leaf. This means the host
formatter controls indentation around the injected span, while the inner
formatter controls the text inside the span.

## Vertical whitespace trimming

The output from pretty printing can leave too much or too few vertical
whitespace (at the end of the file).
This final step clears that up.

<div class="warning">

This trimming happens regardless of whether the whitespace was present
in the input, intentionally or otherwise.

</div>

## Idempotence checking

The Topiary CLI performs "[idempotence][wiki:idempotence] checking" by
default. That is, it checks that formatting (i.e., per the pipeline as
described above) an already-formatted input makes no further changes.

Anecdotally, this incurs a negligible performance penalty to Topiary:
its formatting time is dominated by start-up overhead (e.g., parsing the
query files). However, the check can be disabled; this is often useful
while developing or debugging formatting queries.

> **Note**\
> See the [usage documentation](../cli/usage/format.md) for `topiary
> format` for details on how this is exposed. The [web
> playground](../playground/web.md) also provides this option.

<!-- Links -->
[wiki:idempotence]: https://en.wikipedia.org/wiki/Idempotence
