{
  callPackageNoOverrides,
  advisory-db,
  craneLib,
  prefetchLanguagesFile,
  prefetchLanguagesNickelFile,
  prefetchLanguages,
  generateNcl,
  fromNickelFile,
  topiaryLib,
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
      topiaryLib
      ;
  };
in

{
  inherit topiaryPkgs binPkgs;
}
