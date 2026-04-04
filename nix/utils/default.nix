{
  callPackageNoOverrides,
}:

let
  inherit (callPackageNoOverrides ./nickelUtils.nix { })
    toJSONFile
    fromNickelFile
    toNickelValue
    ;

  inherit
    (callPackageNoOverrides ./prefetchLanguages.nix {
      inherit toJSONFile toNickelValue fromNickelFile;
    })
    prefetchLanguages
    prefetchLanguagesFile
    prefetchLanguagesNickelFile
    ;
in

{
  inherit
    toJSONFile
    fromNickelFile
    prefetchLanguages
    prefetchLanguagesFile
    prefetchLanguagesNickelFile
    ;
}
