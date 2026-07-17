{
  callPackageNoOverrides,
  advisory-db,
  craneLib,
  prefetchLanguagesFile,
  prefetchLanguagesNickelFile,
  prefetchLanguages,
  generateNcl,
  fromNickelFile,
}:

let
  binPkgs = callPackageNoOverrides ./bin.nix { };

  topiaryPkgs = callPackageNoOverrides ./topiary.nix {
    inherit
      advisory-db
      craneLib
      prefetchLanguagesFile
      prefetchLanguagesNickelFile
      prefetchLanguages
      generateNcl
      fromNickelFile
      ;
  };
in

{
  inherit topiaryPkgs binPkgs;
}
