# Language server

`topiary lsp` runs Topiary as a [Language Server Protocol][lsp] (LSP)
server, so editors can format files with Topiary through their built-in
"format document" command. The server reads LSP messages on standard
input and writes responses to standard output; editors start it
themselves, so you won't normally run it by hand.

```
Run Topiary Language Server

Usage: topiary lsp [OPTIONS]

Options:
  -C, --configuration <CONFIGURATION>  Configuration file [env: TOPIARY_CONFIG_FILE]
  -M, --merge-configuration            Enable merging for configuration files
  -v, --verbose...                     Logging verbosity (increased per occurrence)
      --skip-language <LANGUAGE>       Skip formatting the given language(s)
  -h, --help                           Print help
```

## Installation

The language server is an optional feature of the Topiary CLI. It is
not included in the default build, so enable the `lsp` feature when
installing:

```sh
cargo install topiary-cli --features lsp
```

or, from a checkout of the repository:

```sh
cargo install --path topiary-cli --features lsp
```

Run `topiary --help` to check: if `lsp` is not listed among the
commands, your build doesn't include the language server.

## Behaviour

- **Whole-document formatting only.** The server supports
  `textDocument/formatting`. Range formatting, on-type formatting and
  diagnostics are not provided, so Topiary works alongside your usual
  language server rather than replacing it.

- **Same results as `topiary format`.** The language is detected from
  the file extension, exactly as in the CLI, using the same
  [configuration](../configuration.md). Options such as
  `--configuration` and `--skip-language` work as they do for
  `topiary format`, e.g. `topiary lsp --skip-language rust`. [Language
  injections](../../reference/language-injections.md) are formatted too.

- **Files that don't parse are left alone.** If the document contains a
  syntax error, the server refuses to format it and logs an error to
  the editor, rather than risk mangling code you are in the middle of
  editing. Fix the error and format again.

- **Only files on disk.** Documents without a `file://` URI, such as
  unsaved, untitled buffers, are not formatted, as there is no file
  extension to detect the language from.

- **Restart after configuration changes.** Grammars and queries are
  loaded the first time a language is formatted and kept for the
  lifetime of the server. After changing your configuration or query
  files, restart the language server for the changes to take effect.

Errors are reported to the editor as log messages (`window/logMessage`);
where these appear depends on your editor. Passing `-v` (or `-vv`, etc.)
additionally writes Topiary's internal logging to standard error, which
most editors capture in their language server log.

## Editor configuration

### Helix

Define the server and add it to a language in your `languages.toml`.
Helix uses the first language server in the list that supports
formatting, so if the language already has a server that can format
(such as `ocamllsp`), exclude its formatting so that Topiary is used:

```toml
[language-server.topiary]
command = "topiary"
args = ["lsp"]

[[language]]
name = "ocaml"
language-servers = [
  { name = "ocamllsp", except-features = ["format"] },
  "topiary",
]
```

Format with `:format`, or set `auto-format = true` on the language to
format on save.

### Neovim

With Neovim 0.11 or later, the built-in LSP client can be configured
directly:

```lua
vim.lsp.config("topiary", {
  cmd = { "topiary", "lsp" },
  filetypes = { "ocaml", "nickel", "json", "toml" },
  root_markers = { ".git" },
})
vim.lsp.enable("topiary")
```

Adjust `filetypes` to the languages you want Topiary to format, then
format with `vim.lsp.buf.format()`. If another attached server also
provides formatting, Neovim will ask which one to use; pass
`{ name = "topiary" }` to `vim.lsp.buf.format` to pick Topiary directly.

[lsp]: https://microsoft.github.io/language-server-protocol/
