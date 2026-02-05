# Contributing to Topiary

The Topiary team is greatly appreciative of all contributions and will
endeavour to give each the attention it deserves. [Issues](https://github.com/topiary/topiary/issues),
[pull requests](https://github.com/topiary/topiary/pulls) and
[discussions](https://github.com/topiary/topiary/discussions) are all
welcome. If you have any immediate questions, or just want to hang out,
feel free to say "Hi!" on our [Discord channel][discord].

## For language formatter authors

The most effective way of contributing to Topiary is by developing
formatting queries for a language you are interested or invested in that
is not currently supported by Topiary.

The other guides in this section outline the steps needed to bootstrap
this process:

- [Adding a new language](adding-a-new-language.md)
- [Suggested query development workflow](suggested-workflow.md)

More thorough, end-to-end tutorials are also available:

- [Writing a formatter has never been so easy: a Topiary
  tutorial](yann-tutorial.md)

The section below, [for developers](#for-developers), may also be useful
to this end.

### Maturity policy

As described in [Language support](../reference/language-support.md),
formatting queries for languages come in two levels of maturity:
supported and experimental. Cargo feature flags are used to distinguish
these.

Formatting queries from external contributors are also subject to these
levels. However, the Topiary team will not necessarily be familiar with
the language in question and therefore will not be able to accurately
assess the maturity of contributed formatting queries.

In the (current) absence of automated tools that can quantify grammar
coverage of a set of queries, we leave it to contributors to make this
judgement honestly. This can be done by asking yourself the following
question:

> Would I ask a colleague to format their production code with this?

If the answer is "no", because, say, not enough of the language's
grammar is covered, then these queries fall under "experimental"
support. Even if the answer is "maybe", for whatever reason, err on the
side of "experimental"; bearing in mind that, once merged, the Topiary
team will take a best effort approach to fixing any post-contribution
issues, but won't actively maintain these queries.

## For developers

### Nix devshell

A Nix devshell is available, which includes all development
dependencies, for contributing to Topiary. Enter this shell with:

```sh
nix develop
```

### Performance profiling

You can check performance before or after changes by running `cargo
bench`.

If you have [`flamegraph`](https://github.com/flamegraph-rs/flamegraph)
installed, you can also generate a performance flamegraph with, for
example:

```sh
CARGO_PROFILE_RELEASE_DEBUG=true \
cargo flamegraph -p topiary-cli -- \
  format --language ocaml \
  < topiary-cli/tests/samples/input/ocaml.ml \
  > /dev/null
```

This will produce a `flamegraph.svg` plot.

### Code coverage

<div class="warning">
This section has not been updated since December 2022. It may be
out-dated or invalid.

(See issues [#80](https://github.com/topiary/topiary/issues/80) and
[#894](https://github.com/topiary/topiary/issues/894))
</div>

Code coverage metrics can be generated via LLVM profiling data generated
by the build. These can be created by setting appropriate environment
variables to `cargo test`:

```sh
CARGO_INCREMENTAL=0 \
RUSTFLAGS='-Cinstrument-coverage' \
LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' \
cargo test
```

This will build and run the test suite and output
`cargo-test-*-*.profraw` files in the working directory. (Outside of the
Nix devshell, you may need `binutils` installed.)

These files can be used by [`grcov`](https://github.com/mozilla/grcov)
to render a variety of output reports. For example, the following
renders HTML output in `target/coverage/html`:

```sh
grcov \
  --branch \
  --output-type html \
  --source-dir topiary-cli/src \
  --binary-path target/debug/deps \
  --output-path target/coverage/html \
  .
```

<div class="warning">

`grcov` relies on the `llvm-tools-preview` component from `rustup`. For
Nix users, `rustup` can interfere with the Rust toolchain that is
provided by Nix, if you have both installed.

</div>

### Website and web playground

#### Website

If you have [Deno](https://deno.land/) installed, you can start a local
web server like this:

```sh
deno run -A local-web-server.ts
```

The website should then be running on `http://localhost:8080`.

#### Web playground WASM assets

<div class="warning">

The WASM-based web playground is currently _not_ under active
development and has diverged from newer releases of Topiary. Building or
updating the web playground and its associated WASM grammars is **not
likely to function correctly** at this time.

</div>

In order to build or update the web playground, you can run the
following within the Nix devshell:

```sh
update-wasm-app
```

Similarly, to update the Tree-sitter grammar WASM binaries, again within
the Nix devshell, you can run:

```sh
update-wasm-grammars
```

Alternatively, if you have `git`, `tree-sitter` and `emcc` (Emscripten)
in your `PATH`, you can run the `bin/update-wasm-grammars.sh` script
directly.

To use Docker instead, the legacy approach can still be used (using JSON
as an example):

1. Make sure you have Docker running and that you are member of the
   `docker` group, so you can run it without being root.

2. `npm install tree-sitter-cli`, or via some other method.

3. `npm install tree-sitter-json` or Git clone the grammar repository.

   - If you used NPM, `tree-sitter-json` will be fetched under
     `node_modules/tree-sitter-json`.

   - If you used Git, it will be wherever you cloned the repository
     (most likely `tree-sitter-json`).

   Whichever of these options you pick, we will use `GRAMMAR_PATH` as a
   stand-in for the directory where `grammar.js` can be found.

4. Run `npx tree-sitter build-wasm GRAMMAR_PATH`. If you get a Docker
   permission error, you may need to add yourself to the `docker` group.

5. `mv tree-sitter-json.wasm web-playground/public/scripts`

> **Note**\
> Some grammar repositories are slightly different because they can
> contain multiple grammars or grammars under an unconventional path;
> OCaml, for example. In such cases, step 4 (above) should be changed
> such that `GRAMMAR_PATH` points to the directory containing the
> appropriate `grammar.js` file.

#### Web playground frontend

The playground frontend is a small React app. You can run a development
server with the following:

```sh
cd web-playground
npm install
npm run dev
```

If you want to build the playground so it works with the full website
running with Deno, as [above](#website), you can now just do `npm run
build`.

<!-- Links -->
[discord]: https://discord.gg/FSnkvNyyzC
