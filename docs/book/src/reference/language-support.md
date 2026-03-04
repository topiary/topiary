# Language support

<!----------------------------------------------------------------------
Update these sections as necessary on new developments/releases
----------------------------------------------------------------------->

Topiary ships with formatting support for a number of languages. The
formatting styles for these languages come in two levels of maturity:

- **Supported** languages are actively maintained by the Topiary team.

- **Experimental** languages do not cover a significant proportion of
  the target grammar, may contain formatting bugs and could even be
  deprecated. You should not use these to format production code.

We also ship formatting styles for languages from a number of
contributors. The Topiary team does not actively maintain these and --
while not necessarily -- where indicated, they should also be considered
experimental.

## Supported

These formatting styles cover their target language and fulfil Topiary's
stated design goals. They are exposed, in Topiary, through the
`--language` command line flag, or language detection (based on file
extension).

- [Bash]
- [JSON]
- [JSON5]
- [Nickel]
- [OCaml] (both implementations and interfaces)
- [OCamllex]
- [TOML]
- [Tree Sitter Queries][tree-sitter-query]

## Contributed

These languages' formatting styles have been generously provided by
external contributors. They are built in, by default -- unless marked as
experimental -- so are exposed in the same way as supported languages.

- [CSS], by [Eric Lavigne](https://github.com/lavigneer)
- [OpenSCAD], by [Mikhail Katychev](https://github.com/mkatychev)
- [SDML], by [Simon Johnston](https://github.com/johnstonskj)
- [WIT], by [Mikhail Katychev](https://github.com/mkatychev)

## Experimental

These languages' formatting styles -- from either the Topiary team or
external contributors -- are subject to change and/or not yet considered
production-ready. They are _not_ built by default and are gated behind a
feature flag (either `experimental`, for all of them, or by their
individual name). Once included, they can be accessed in Topiary in the
usual way.

- [Rust]

# Topiary cultivars

Topiary is being used elsewhere, either as a fork or a
[library](library.md), to create formatters for other languages. These
are the ones we know about!

- **[topiary-nushell]**, by @blindFS \
  A formatter for the [Nushell] shell language.

- [**Godot GDScript Formatter**][gdscript-formatter], by GDQuest \
  A formatter for Godot's [GDScript] programming language.

<div class="warning">

If you have your own Topiary, which you'd like to have featured here,
please do not hesitate to get in touch with us on our [Discord
channel][discord].

</div>

<!-- Links -->
[bash]: https://www.gnu.org/software/bash
[css]: https://en.wikipedia.org/wiki/CSS
[discord]: https://discord.gg/FSnkvNyyzC
[gdscript-formatter]: https://gdquest.com/library/gdscript_formatter
[gdscript]: https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/index.html
[json]: https://www.json.org
[nickel]: https://nickel-lang.org
[nushell]: https://nushell.sh
[ocaml]: https://ocaml.org
[ocamllex]: https://v2.ocaml.org/manual/lexyacc.html
[openscad]: https://en.wikipedia.org/wiki/OpenSCAD
[rust]: https://www.rust-lang.org
[sdml]: https://sdml.io/
[toml]: https://toml.io
[topiary-nushell]: https://github.com/blindFS/topiary-nushell/
[tree-sitter-query]: https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries
[wit]: https://component-model.bytecodealliance.org/design/wit.html
