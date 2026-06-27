{
  callPackageNoOverrides,
  topiaryUtils,
  topiary-cli,
}:

let
  inherit
    (callPackageNoOverrides ./config.nix {
      inherit (topiaryUtils) fromNickelFile toJSONFile prefetchLanguages;
    })
    defaultConfig
    defaultConfigFile
    defaultConfigPrefetched
    defaultConfigPrefetchedFile
    wrapWithConfig
    wrapWithConfigFile
    ;

  inherit
    (callPackageNoOverrides ./gitHook.nix {
      inherit topiary-cli defaultConfigPrefetched wrapWithConfig;
    })
    gitHookFor
    gitHook
    gitHookBinFor
    gitHookBin
    ;
in

{
  inherit
    defaultConfig
    defaultConfigFile
    defaultConfigPrefetched
    defaultConfigPrefetchedFile
    wrapWithConfig
    wrapWithConfigFile
    gitHookFor
    gitHook
    gitHookBinFor
    gitHookBin
    ;
}
