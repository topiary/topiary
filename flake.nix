{
  description = "A general code formatter based on tree-sitter.";

  nixConfig = {
    extra-substituters = [
      "https://tweag-topiary.cachix.org"
    ];
    extra-trusted-public-keys = [
      "tweag-topiary.cachix.org-1:8TKqya43LAfj4qNHnljLpuBnxAY/YwEBfzo3kzXxNY0="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    rust-overlay.url = "github:oxalica/rust-overlay";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    { nixpkgs, ... }@inputs:
    let
      forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;

      # NOTE: This flake is only a wrapper providing dependencies lock and
      # interface. All the logic lives in the `nix` directory. We import it here
      # bundled for all systems as a `topiaryNix`.
      topiaryNix = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        import ./nix {
          inherit pkgs;
          inherit (inputs) advisory-db crane rust-overlay;
        }
      );

    in
    {
      packages = forAllSystems (system: topiaryNix.${system}.packages);
      lib = forAllSystems (system: topiaryNix.${system}.lib);
      checks = forAllSystems (system: topiaryNix.${system}.checks);
      devShells = forAllSystems (system: topiaryNix.${system}.devShells);

      homeManagerModules = rec {
        topiary = import ./nix/modules/home-manager.nix { inherit topiaryNix; };
        default = topiary;
      };
    };
}
