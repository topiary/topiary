{
  lib,
  stdenv,
  writeShellApplication,

  inotify-tools,
  emscripten,
  git,
  nickel,
  tree-sitter,
  diffutils,
  gnused,
  nixdoc,
  jq,
}:

let
  inherit (builtins)
    readFile
    ;

  # FIXME: Broken
  # TODO: Don't use rustup to install these components but just use Nix
  # generate-coverage = writeShellApplication {
  #   name = "generate-coverage";

  #   runtimeInputs = [
  #     cacert
  #     grcov
  #     rustup
  #   ];

  #   text = readFile ../../bin/generate-coverage.sh;
  # };

  generate-nix-documentation = writeShellApplication {
    name = "generate-nix-documentation";
    runtimeInputs = [ nixdoc ];
    text = readFile ../../bin/generate-nix-documentation.sh;
  };

  playground = writeShellApplication {
    name = "playground";

    runtimeInputs = lib.optionals (!stdenv.isDarwin) [
      inotify-tools
    ];

    text = readFile ../../bin/playground.sh;
  };

  verify-documented-usage = writeShellApplication {
    name = "verify-documented-usage";

    runtimeInputs = [
      diffutils
      gnused
    ];

    text = readFile ../../bin/verify-documented-usage.sh;
  };

in
{
  inherit
    generate-nix-documentation
    playground
    verify-documented-usage
    ;
}
