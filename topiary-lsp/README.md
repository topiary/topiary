# topiary-lsp

Language Server Protocol (LSP) frontend for [Topiary](https://github.com/tweag/topiary), the universal code formatter.

## Installation

`topiary-lsp` is integrated into the main `topiary` CLI.

```sh
cargo install topiary-cli
```

You can then run the LSP server using:
```sh
topiary lsp
```

## Editor Configuration

### Helix

To use `topiary-lsp` with [Helix](https://helix-editor.com/), add it to your `languages.toml`.

For example, to configure it as the formatter for OCaml:

```toml
[language-server.topiary-lsp]
command = "topiary"
args = ["lsp"]

[[language]]
name = "ocaml"
language-servers = ["topiary-lsp"]
```
