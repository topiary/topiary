## Generate a Topiary `languages.ncl` configuration from a Nix attribute set.
##
## This complements `prefetchLanguages.nix`, which takes an existing `.ncl`
## file and replaces its grammar sources. Here we go the other direction: the
## source of truth lives in Nix, and we render Nickel source from it,
## preserving the `| default` annotations expected by Topiary.

{
  lib,
  writeText,
  toNickelValue,
}:

let
  inherit (builtins) concatStringsSep;
  inherit (lib.attrsets) mapAttrsToList;

  ## HACK: Nickel cannot export to its own source format (no `nickel export
  ## --format nickel`), so we reconstruct Nickel source from Nix manually,
  ## re-adding `| default` annotations where requested. If the per-language
  ## schema grows new fields, this function must be updated to match.

  ## NOTE on `| default`: Topiary always merges the user-supplied config
  ## (`-C`) with the embedded built-in (see `Configuration::fetch` in
  ## `topiary-config/src/lib.rs`). If both sides annotate `grammar.source`
  ## with `| default`, the records get merged field-wise into something like
  ## `{ git = {...}, path = "..." }` -- multiple keys -- which fails the
  ## externally-tagged `GrammarSource` enum deserialization. So `| default`
  ## should be emitted only when the generated file is meant to act as a
  ## default that other configs can override (e.g. as the embedded built-in
  ## via `include_str!`). When it will be passed via `-C`, omit them.

  /**
    Render a single language config as Nickel source.

    # Type

    ```
    languageToNickel : Bool -> String -> LanguageConfig -> String
    ```
  */
  languageToNickel =
    withDefaults: name: lang:
    let
      ann = if withDefaults then " | default" else "";
      fields = [
        "extensions${ann} = ${toNickelValue lang.extensions}"
      ]
      ++ lib.optional (lang ? indent) "indent${ann} = ${toNickelValue lang.indent}"
      ++ [
        (
          let
            grammarFields = [
              "source${ann} = ${toNickelValue lang.grammar.source}"
            ]
            ++ lib.optional (lang.grammar ? symbol) "symbol = ${toNickelValue lang.grammar.symbol}";
          in
          "grammar = { ${concatStringsSep ", " grammarFields} }"
        )
      ];
    in
    "${name} = {\n      ${concatStringsSep ",\n      " fields},\n    }";

  /**
    Convert a Topiary configuration (as a Nix attribute set with the same
    shape as `languages.ncl`) to a `.ncl` file usable as Topiary's
    configuration.

    `withDefaults` controls whether `| default` annotations are emitted. Set
    it to `true` when generating a file that is meant to be the *embedded*
    default config (i.e. drop-in replacement for the bundled
    `topiary-config/languages.ncl`); set it to `false` when the file will be
    supplied at runtime via `-C` / `TOPIARY_CONFIG_FILE`, where Topiary
    merges it with the built-in default and conflicting `| default` records
    would produce malformed multi-key sources.

    # Type

    ```
    generateNcl : { name : String, config : TopiaryConfig, withDefaults ? Bool } -> File
    ```
  */
  generateNcl =
    {
      name,
      config,
      withDefaults ? true,
    }:
    let
      body = concatStringsSep ",\n\n    " (
        mapAttrsToList (languageToNickel withDefaults) config.languages
      );
    in
    writeText name ''
      {
        languages = {
          ${body},
        }
      }
    '';
in
{
  inherit generateNcl languageToNickel;
}
