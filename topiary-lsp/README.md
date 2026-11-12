# topiary-lsp

Language Server Protocol (LSP) frontend for [Topiary](https://github.com/topiary/topiary), the universal code formatter.

## Installation

`topiary-lsp` is integrated into the main `topiary` CLI, but disabled by default.

```sh
cargo install topiary-cli --features lsp
```

You can then run the LSP server using:
```sh
topiary lsp
```

## Editor Configuration

### Helix/Mitos

To use `topiary-lsp` with [Helix](https://helix-editor.com/) or [Mitos](https://mitos.computer/), add it to your `languages.toml`.

For example, to configure it as the formatter for OCaml:

```toml
[language-server.topiary-lsp]
command = "topiary"
args = ["lsp"]

[[language]]
name = "ocaml"
language-servers = ["topiary-lsp"]
```
