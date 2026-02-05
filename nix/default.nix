{
  pkgs ? import <nixpkgs> { },

  # The following dependencies are hard to provide without using flakes, making
  # our “you don't have to use flakes” claim deceptive. We keep it like this for
  # now, and if someone complains we can find another solution. Context in
  # https://github.com/topiary/topiary/pull/1026#discussion_r2131778007
  advisory-db,
  crane,
  rust-overlay,
}:

let
  overlays = import ./overlays;

  pkgs' = pkgs.appendOverlays [
    overlays.wasm-bindgen-cli
    rust-overlay.overlays.default
  ];

  # A simpler version of `callPackage` that only works on files and does not
  # rely on `makeOverridable`, to avoid polluting the output.
  callPackageNoOverrides =
    file: args:
    let
      fn = import file;
      auto-args = builtins.intersectAttrs (builtins.functionArgs fn) pkgs';
      final-args = auto-args // args;
    in
    fn final-args;

  craneLib = crane.mkLib pkgs';

  topiaryUtils = callPackageNoOverrides ./utils {
    inherit callPackageNoOverrides;
  };

  inherit
    (callPackageNoOverrides ./packages {
      inherit
        advisory-db
        craneLib
        callPackageNoOverrides
        ;
      inherit (topiaryUtils) prefetchLanguagesFile;
    })
    topiaryPkgs
    binPkgs
    ;

  # NOTE: The name could clashes with nixpkgs' lib, which could lead to
  # unexpected behaviours in subsequent `callPackage` statements.
  topiaryLib = callPackageNoOverrides ./lib {
    inherit (topiaryPkgs) topiary-cli;
    inherit callPackageNoOverrides topiaryUtils;
  };

  checks = callPackageNoOverrides ./checks {
    inherit (pkgs') emptyFile;
    inherit topiaryPkgs;
    inherit (topiaryLib) gitHook;
  };

  devShells = callPackageNoOverrides ./devShells {
    inherit
      checks
      craneLib
      binPkgs
      topiaryPkgs
      ;
  };

in
{
  # REVIEW: I have kept the separation between “Topiary” vs. “Bin” packages,
  # and I plug it throughout the Nix code, only to flatten it at the very end
  # because that's what flakes want. Maybe we want a flatter organisation from
  # the get-go, where everything gets merged in `packages/default.nix` and we
  # only manipulate the `packages` set?
  packages = topiaryPkgs // binPkgs;
  lib = topiaryUtils // topiaryLib;
  inherit checks devShells;
}
