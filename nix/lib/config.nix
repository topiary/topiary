{
  writeShellApplication,
  fromNickelFile,
  toJSONFile,
  prefetchLanguages,
}:

let
  /**
    Same as `defaultConfig` but as a Nickel File.

    # Type

    ```
    defaultConfigFile : File
    ```
  */
  defaultConfigFile = ../../topiary-config/languages.ncl;

  /**
    The default configuration of Topiary -- that is the content of
    `topiary-config/languages.ncl` -- but as a Nix value.

    # Type

    ```
    defaultConfig : TopiaryConfig
    ```
  */
  defaultConfig = fromNickelFile defaultConfigFile;

  /**
    Same as `defaultConfig`, but where all the language grammars point to the
    Nix store path of a prefetched and precompiled version of the grammar. This
    is `prefetchLanguages defaultConfig`.

    # Type

    ```
    defaultConfigPrefetched
    ```
  */
  defaultConfigPrefetched = prefetchLanguages defaultConfig;

  /**
    Same as `defaultConfigPrefetched` but as a JSON file, consumable by Nickel.

    # Type

    ```
    defaultConfigFilePrefetched : File
    ```
  */
  defaultConfigFilePrefetched = toJSONFile "languages-prefetched.json" defaultConfigPrefetched;

  /**
    Same as `wrapWithConfig` but with the configuration as a Nickel file.

    # Type

    ```
    wrapWithConfigFile : Derivation -> File -> Derivation
    ```
  */
  wrapWithConfigFile =
    { package, configFile }:
    writeShellApplication {
      name = "topiary";
      text = ''exec ${package}/bin/topiary -C ${configFile} "$@"'';
    };

  /**
    Given a derivation providing `/bin/topiary` and a Topiary configuration
    `<config>`, produce an executable running `topiary -C <config>`.
    Note that this is different from building Topiary with a different default
    configuration: the resulting binary will not accept an additional `-C`
    argument.

    # Type

    ```
    wrapWithConfig : Derivation -> TopiaryConfig -> Derivation
    ```
  */
  wrapWithConfig =
    { package, config }:
    wrapWithConfigFile {
      inherit package;
      configFile = toJSONFile "languages.json" config;
    };

in
{
  inherit
    defaultConfig
    defaultConfigPrefetched
    defaultConfigFile
    defaultConfigFilePrefetched
    wrapWithConfig
    wrapWithConfigFile
    ;
}
