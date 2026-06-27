# Configuration

Topiary is configured using `languages.ncl` files. The `.ncl` extension
relates to [Nickel](https://nickel-lang.org/), a configuration language
created by Tweag. There are up to four sources where Topiary checks for
such a file.

## Configuration sources

At build time the
[`languages.ncl`](https://github.com/topiary/topiary/blob/main/topiary-config/languages.ncl)
in the root of the Topiary repository is embedded into Topiary. This
file is parsed at runtime. The purpose of this `languages.ncl` file is
to provide sane defaults for users of Topiary (both the library and the
CLI binary).

The next two are read by the Topiary binary at runtime and allow the
user to configure Topiary to their needs. The first is intended to be
user specific, and can thus be found in the configuration directory of
the OS:

| OS      | Typical Configuration Path                                       |
| :------ | :--------------------------------------------------------------- |
| Unix    | `/home/alice/.config/topiary/languages.ncl`                      |
| macOS   | `/Users/Alice/Library/Application Support/Topiary/languages.ncl` |
| Windows | `C:\Users\Alice\AppData\Roaming\Topiary\config\languages.ncl`    |

> **Note**\
> On macOS, Topiary also looks in `~/.config/topiary`, as well as the
> standard macOS configuration directory.

This file is not automatically created by Topiary.

The next source is intended to be a project-specific settings file for
Topiary. When running Topiary in some directory, it will ascend the file
tree until it finds a `.topiary` directory. It will then read any
`languages.ncl` file present in that directory.

Finally, an explicit configuration file may be specified using the
`-C`/`--configuration` command line argument (or the
`TOPIARY_CONFIG_FILE` environment variable). This is intended for
driving Topiary under very specific use-cases.

To summarise, Topiary consumes configuration from these sources in the
following order (highest to lowest):

1. The explicit configuration file specified as a CLI argument.
2. The project-specific Topiary configuration.
3. The user configuration file in the OS's configuration directory.
4. The built-in configuration file.

One can call `topiary config show-sources` to display configuration
sources along with any detected `language.ncl` files or `queries`
directories therein:

```
$ topiary config show-sources

╭───────────┬──────────────────────────────────────────────────────┬───────────────┬─────────╮
│ source    │ path                                                 │ languages.ncl │ queries │
├───────────┼──────────────────────────────────────────────────────┼───────────────┼─────────┤
│ workspace │ /Users/USER_NAME/Documents/rust/topiary/.topiary     │ ✗             │ ✗       │
├───────────┼──────────────────────────────────────────────────────┼───────────────┼─────────┤
│ unix-home │ /Users/USER_NAME/.config/topiary                     │ ✓             │ ✓       │
├───────────┼──────────────────────────────────────────────────────┼───────────────┼─────────┤
│ OS        │ /Users/USER_NAME/Library/Application Support/topiary │ ✗             │ ✗       │
├───────────┼──────────────────────────────────────────────────────┼───────────────┼─────────┤
│ built-in  │ <built-in>                                           │ ✓             │ ✓       │
╰───────────┴──────────────────────────────────────────────────────┴───────────────┴─────────╯
```

> **Note**\
>  Only the highest priority `queries` directory will be used.

### Configuration merging

By default, Topiary only considers the configuration file with the
highest priority and merges it with the built-in configuration. However,
if the `-M`/`--merge-configuration` option is provided to the CLI, then
all available configurations are merged together, as per the [Nickel
specification](https://nickel-lang.org/user-manual/merging).

In which case, if one of the sources listed above attempts to define a
language configuration already present in the built-in configuration, or
if two configuration files have conflicting values, then Topiary will
display a Nickel error.

To understand why, one can read the [Nickel documentation on
merging](https://nickel-lang.org/user-manual/merging). However, the
short answer is that a priority must be defined. The built-in
configuration has everything defined with priority 0. Any priority above
that will replace any other priority. For example, to override the
entire Bash configuration, use the following Nickel file.

```nickel
{
  languages = {
    # Alternatively, use `priority 1`, rather than `force`
    bash | force = {
      extensions = [ "sh" ],
      indent = "    ",
    },
  },
}
```

To override only the indentation, use the following Nickel file:

```nickel
{
  languages = {
    bash = {
      indent | force = "    ",
    },
  },
}
```

<div class="warning">

The merging semantics for Topiary's grammar configuration (see
[below](#specifying-the-grammar)) is not yet fully defined; see issue
[#861](https://github.com/topiary/topiary/issues/861).

</div>

## Configuration options

The configuration file contains a record of languages. That is, it
defines language identifiers and configures them accordingly.

For instance, the configuration for Nickel is defined as such:

```nickel
nickel = {
  extensions = ["ncl"],
},
```

The language identifier is used by Topiary to associate the language
entry with the respective query file, as well exposing it to the user
through the `--language` CLI argument. This value should be written in
lowercase.

### File extensions

The list of extensions is mandatory for every language, but does not
necessarily need to exist in every configuration file. It is sufficient
if, for every language, there is a single configuration file that
defines the list of extensions for that language.

### Indentation

The optional field, `indent`, exists to define the indentation method
for that language. Topiary defaults to two spaces `"  "` if it cannot
find the indent field in any configuration file for a specific language.

### Specifying the grammar

Topiary fetches and builds the grammar for you, or a grammar can be
provided by some other method. To have Topiary fetch the grammar for
you, specify the `grammar.source.git` attribute of a language:

```nickel
nickel = {
  extensions = ["ncl"],
  grammar.source.git = {
    git = "https://github.com/nickel-lang/tree-sitter-nickel",
    rev = "43433d8477b24cd13acaac20a66deda49b7e2547",
  },
},
```

To specify a prebuilt grammar, specify the `grammar.source.path`
attribute, which must point to a compiled grammar file on your file
system:

```nickel
nickel = {
  extensions = ["ncl"],
  grammar.source.path = "/path/to/compiled/grammar/file.so",
},
```

> **Note**\
> If you want to link to a grammar file that has already been compiled
> by Topiary itself, those look like `~/.cache/topiary/<LANGUAGE>/<GIT_HASH>.so`
> (or the equivalent for your platform).

For usage in Nix, a `prefetchLanguages.nix` file provides utilities
allowing to transform a Topiary configuration into one where languages
have been pre-fetched and pre-compiled in Nix derivations. The only
caveat is that, for each Git source, the configuration must contain a
`nixHash` for that source. For instance:

```nickel
nickel = {
  extensions = ["ncl"],
  grammar.source.git = {
    git = "https://github.com/nickel-lang/tree-sitter-nickel",
    rev = "43433d8477b24cd13acaac20a66deda49b7e2547",
    nixHash = "sha256-9Ei0uy+eGK9oiH7y2KIhB1E88SRzGnZinqECT3kYTVE=",
  },
},
```

The simplest way to obtain the hash is to use `nix-prefetch-git` (and
look for the `hash` field in its output):

```sh
nix run nixpkgs#nix-prefetch-git -- https://github.com/bytecodealliance/tree-sitter-wit 230984dfaf803a0ff8f77da5034361a62c326577
```

The second simplest way is to compile, which will show something like:

```
evaluation warning: Language `nickel`: no nixHash provided - using dummy value
error: hash mismatch in fixed-output derivation '/nix/store/jgny7ll7plh7rfdnvdpgcb82kd51aiyx-tree-sitter-nickel-43433d8.drv':
         specified: sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
            got:    sha256-9Ei0uy+eGK9oiH7y2KIhB1E88SRzGnZinqECT3kYTVE=
error: 1 dependencies of derivation '/nix/store/0q20rk8l4g0n5fzr0w45agxx0j9qy65v-nickel-grammar-43433d8477b24cd13acaac20a66deda49b7e2547.drv' failed to build
error: 1 dependencies of derivation '/nix/store/s5phxykjyzqay7gc33hc6f8kw4ndba25-languages-prefetched.json.drv' failed to build
error: 1 dependencies of derivation '/nix/store/5w15p3b3xfw5nd6mxz58ln09v10kvf8v-languages-prefetched.ncl.drv' failed to build
error: 1 dependencies of derivation '/nix/store/7zzyha67jw09kc37valp28bp5h6i7dka-topiary-0.6.0.drv' failed to build
```
