{
  callPackageNoOverrides,
  advisory-db,
  craneLib,
  prefetchLanguagesFile,
  prefetchLanguagesNickelFile,
}:

let
  binPkgs = callPackageNoOverrides ./bin.nix { };

  topiaryPkgs = callPackageNoOverrides ./topiary.nix {
    inherit
      advisory-db
      craneLib
      prefetchLanguagesFile
      prefetchLanguagesNickelFile
      ;
  };
in

{
  inherit topiaryPkgs binPkgs;
}
