{
  callPackageNoOverrides,
  advisory-db,
  craneLib,
  prefetchLanguagesFile,
  prefetchLanguagesNickelFile,
  prefetchLanguages,
  generateNcl,
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
      ;
  };
in

{
  inherit topiaryPkgs binPkgs;
}
