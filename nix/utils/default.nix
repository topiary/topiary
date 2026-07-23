{
  callPackageNoOverrides,
}:

let
  inherit (callPackageNoOverrides ./nickelUtils.nix { })
    toJSONFile
    fromNickelFile
    toNickelValue
    ;

  inherit (callPackageNoOverrides ./generate-ncl.nix { inherit toNickelValue; })
    generateNcl
    languageToNickel
    ;

  inherit
    (callPackageNoOverrides ./prefetchLanguages.nix {
      inherit toJSONFile fromNickelFile generateNcl;
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
    generateNcl
    languageToNickel
    prefetchLanguages
    prefetchLanguagesFile
    prefetchLanguagesNickelFile
    ;
}
