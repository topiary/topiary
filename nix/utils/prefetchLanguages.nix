## Because of dynamic loading, Topiary plays poorly in the Nix sandbox. This
## file introduces two utilities, `prefetchLanguages` and
## `prefetchLanguagesFile` that transform a Topiary configuration into another
## one where all the grammars have been pre-fetched and pre-compiled in Nix
## derivations.

{
  lib,
  fetchgit,
  nickel,
  runCommand,
  writeText,
  tree-sitter,
  toJSONFile,
  toNickelValue,
  fromNickelFile,
}:

let
  inherit (builtins)
    attrNames
    concatStringsSep
    mapAttrs
    toFile
    readFile
    toJSON
    fromJSON
    baseNameOf
    isList
    isString
    isAttrs
    ;
  inherit (lib) warn;
  inherit (lib.strings) removeSuffix;
  inherit (lib.attrsets) updateManyAttrsByPath;
  inherit (lib.attrsets) mapAttrsToList;

  prefetchLanguageSourceGit =
    name: source:
    tree-sitter.buildGrammar {
      language = name;
      version = source.rev;
      src = fetchgit {
        url = source.git;
        rev = source.rev;
        hash =
          if source ? "nixHash" then
            source.nixHash
          else
            warn "Language `${name}`: no nixHash provided - using dummy value" "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
      };
      location = if source ? "subdir" then source.subdir else null;
    };

  prefetchLanguageSource =
    name: source:
    if source ? "path" then
      { inherit (source) path; }
    else if source ? "git" then
      { path = "${prefetchLanguageSourceGit name source.git}/parser"; }
    else
      throw ("Unsupported Topiary language sources: " ++ concatStringsSep ", " (attrNames source));

  updateByPath = path: update: updateManyAttrsByPath [ { inherit path update; } ];

  /**
    Given a Topiary configuration as a Nix value, returns the same
    configuration, except all language sources have been replaced by a
    prefetched and precompiled one. This requires the presence of a `nixHash`
    for all sources.

    # Type

    ```
    prefetchLanguages : TopiaryConfig -> TopiaryConfig
    ```
  */
  prefetchLanguages = updateByPath [ "languages" ] (
    mapAttrs (name: updateByPath [ "grammar" "source" ] (prefetchLanguageSource name))
  );

  /**
    Same as `prefetchLanguages`, but expects a path to a Nickel file, and
    produces a path to a JSON file, which can be consumed by Nickel.

    # Type

    ```
    prefetchLanguagesFile : File -> File
    ```
  */
  prefetchLanguagesFile =
    topiaryConfigFile:
    toJSONFile "${removeSuffix ".ncl" (baseNameOf topiaryConfigFile)}-prefetched.json" (
      prefetchLanguages (fromNickelFile topiaryConfigFile)
    );

  # Convert a single language config to Nickel source with | default annotations
  languageToNickel =
    name: lang:
    let
      fields = [
        "extensions | default = ${toNickelValue lang.extensions}"
      ]
      ++ lib.optional (lang ? indent) "indent | default = ${toNickelValue lang.indent}"
      ++ [
        (
          let
            grammarFields = [
              "source | default = ${toNickelValue lang.grammar.source}"
            ]
            ++ lib.optional (lang.grammar ? symbol) "symbol = ${toNickelValue lang.grammar.symbol}";
          in
          "grammar = { ${concatStringsSep ", " grammarFields} }"
        )
      ];
    in
    "${name} = {\n      ${concatStringsSep ",\n      " fields},\n    }";

  ## HACK: The following function exists because Nickel has no native way to
  ## export/convert to its own source format. We therefore reconstruct Nickel
  ## source from a Nix attribute set, manually re-adding `| default`
  ## annotations. This is fragile: if the shape of the Topiary configuration
  ## changes (e.g., new per-language fields), this function must be updated to
  ## match. Ideally, Nickel would support `--format nickel` in `nickel export`,
  ## which would make all of this unnecessary.

  /**
    Same as `prefetchLanguages`, but expects a path to a Nickel file, and
    produces a path to a Nickel file with `| default` annotations preserved.
    This can be used as a drop-in replacement for the original `languages.ncl`,
    including when embedded via `include_str!`.

    # Type

    ```
    prefetchLanguagesNickelFile : File -> File
    ```
  */
  prefetchLanguagesNickelFile =
    topiaryConfigFile:
    let
      config = prefetchLanguages (fromNickelFile topiaryConfigFile);
      body = concatStringsSep ",\n\n    " (mapAttrsToList languageToNickel config.languages);
    in
    writeText "${removeSuffix ".ncl" (baseNameOf topiaryConfigFile)}-prefetched.ncl" ''
      {
        languages = {
          ${body},
        }
      }
    '';

in
{
  inherit
    prefetchLanguages
    prefetchLanguagesFile
    prefetchLanguagesNickelFile
    ;
}
