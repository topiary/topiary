# Web playground

<div class="warning">

As of October 2024, the WASM-based web playground is _not_ under active
development and has diverged from newer releases of Topiary. The web
playground is kept online, for demonstrative purposes, but newer
features have **not been implemented**.

See GitHub to [assess the divergence](https://github.com/topiary/topiary/compare/playground...main).

</div>

The [Topiary web playground](https://topiary.tweag.io/playground) is a
browser-based tool for experimenting with Topiary. It provides three
editor panes:

1. **Query:** This editor pane lets you define the Topiary formatting
   queries to format the given language.

2. **Input:** This editor pane allows you to set the input code in the
   given language.

3. **Output:** This editor pane (which is read only) shows the result of
   Topiary formatting (i.e., the defined queries run against the input).

The given language is set by a drop-down at the top of the window.
Changing this will set the appropriate Tree-sitter grammar and populate
the "Query" and "Input" panes with the default formatting queries for
that language, as shipped with Topiary, and the sample testing input for
that language; the "Output" pane will then be generated appropriately.

By default, the web playground will format the input on-the-fly (i.e.,
on changes to either the "Query" or "Input" panes). This option can be
disabled with the appropriate checkbox and the "Format" button can be
used as an alternative. Otherwise, options are provided that mimic the
`--skip-idempotence` and `--tolerate-parsing-errors` flags to the
Topiary CLI (see [`topiary format`](../cli/usage/format.md)).
