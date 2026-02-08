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
    ;
  inherit (lib) warn;
  inherit (lib.strings) removeSuffix;
  inherit (lib.attrsets) updateManyAttrsByPath;

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

in
{
  inherit
    prefetchLanguages
    prefetchLanguagesFile
    ;
}
