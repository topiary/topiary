{
  pkgs,
  checks ? { },
  craneLib,
  binPkgs,
  topiaryPkgs ? { },
  includeExtraPackages ? true,
}:

craneLib.devShell (
  {
    inherit checks;
  }
  // (
    if includeExtraPackages then
      {
        packages =
          with pkgs;
          with binPkgs;
          [
            cargo-dist
            cargo-flamegraph
            rust-analyzer

            jq
            nixdoc

            topiaryPkgs.mdbook
            mdbook-man
            topiaryPkgs.mdbook-manmunge

            pkg-config
            openssl.dev

            # Our own scripts
            # FIXME: Broken
            # generate-coverage
            generate-nix-documentation
            verify-documented-usage
          ]
          ++ pkgs.lib.optionals (!stdenv.isDarwin) [
            playground
          ];
      }
    else
      { }
  )
)
