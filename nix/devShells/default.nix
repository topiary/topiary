{
  callPackage,
  checks,
  craneLib,
  binPkgs,
  topiaryPkgs,
}:

{
  default = callPackage ./devShell.nix {
    inherit
      craneLib
      binPkgs
      checks
      topiaryPkgs
      ;
  };

  minimal = callPackage ./devShell.nix {
    checks = {
      inherit (checks)
        clippy
        fmt
        topiary-core
        topiary-cli
        ;
    };
    inherit craneLib binPkgs topiaryPkgs;
    includeExtraPackages = false;
  };
}
