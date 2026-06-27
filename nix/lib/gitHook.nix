{
  lib,
  topiary-cli,
  wrapWithConfig,
  defaultConfigPrefetched,
}:

let
  inherit (lib)
    concatMap
    attrValues
    concatStringsSep
    filter
    filterAttrs
    elem
    ;

  /**
    This function filters a Topiary configuration (as a Nix value) to include or
    exclude the given languages. See `gitHookFor` for a more thorough
    description of the arguments.

    # Type

    ```
    filterConfig : AttrSet -> AttrSet
    ```
  */
  filterConfig =
    {
      config ? defaultConfigPrefetched,
      includeLanguages ? null, # `null` means all languages
      excludeLanguages ? [ ],
      ...
    }:
    let
      ## Language names that are listed in `includeLanguages` or
      ## `excludeLanguages` but absent from `config.languages`.
      unsupportedLanguages = filter (lang: !(config.languages ? ${lang})) (
        (if includeLanguages == null then [ ] else includeLanguages) ++ excludeLanguages
      );
    in
    if includeLanguages != null && excludeLanguages != [ ] then
      throw "gitHook: cannot pass both `includeLanguages` and `excludeLanguages`."
    else if unsupportedLanguages != [ ] then
      throw "gitHook: unsupported languages: ${concatStringsSep ", " unsupportedLanguages}."
    else if includeLanguages != null then
      config // { languages = filterAttrs (lang: _: elem lang includeLanguages) config.languages; }
    else
      config // { languages = filterAttrs (lang: _: !(elem lang excludeLanguages)) config.languages; };

  /**
    This function exposes the derivation providing `/bin/topiary` in `gitHookFor`.
    It is meant for clients wanting to get a Git hook but also acquire the same
    Topiary, for instance to expose it in their environment. See `gitHookFor` for a
    description of the arguments.

    # Type

    ```
    gitHookBinFor : AttrSet -> Derivation
    ```
  */
  gitHookBinFor =
    inputs@{
      package ? topiary-cli,
      ...
    }:
    wrapWithConfig {
      inherit package;
      config = filterConfig inputs;
    };

  /**
    A Git hook compatible with https://github.com/cachix/git-hooks.nix, and that
    runs `topiary format` on the staged files.

    # Type

    ```
    gitHookFor : AttrSet -> AttrSet
    ```

    # Arguments

    package
    : A derivation providing `topiary-cli`. Defaults to the one shipped with
      this library.

    config
    : The Topiary configuration to use; defaults to a prefetched version of
      `topiary-config/languages.ncl` shipped with this library.

    includeLanguages
    : A list of languages to include from the configuration. The hook will not
      process the others. All the languages in this list must be present in
      `config.languages`. Defaults to `null`, which represents all the languages
      of the configuration.

    excludeLanguages
    : A list of languages to exclude from the hook. One cannot use both
      `includeLanguages` and `excludeLanguages`. All the languages in this list
      must be present in `config.languages`. Defaults to `[]`.
  */
  gitHookFor = inputs: {
    enable = false;
    name = "topiary";
    description = "A general code formatter based on tree-sitter.";
    entry = "${gitHookBinFor inputs}/bin/topiary format";
    files =
      let
        extensions = concatMap (c: c.extensions) (attrValues (filterConfig inputs).languages);
      in
      "\\.(" + concatStringsSep "|" extensions + ")$";
  };

  /**
    ```
    gitHook : AttrSet
    ```

    Same as `gitHookFor {}`.
  */
  gitHook = gitHookFor { };

  /**
    ```
    gitHookBin : Derivation
    ```

    Same as `gitHookBinFor {}`.
  */
  gitHookBin = gitHookBinFor { };

in
{
  inherit
    gitHookFor
    gitHookBinFor
    gitHook
    gitHookBin
    ;
}
