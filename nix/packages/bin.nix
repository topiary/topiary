{
  lib,
  stdenv,
  writeShellApplication,
  writeTextFile,

  inotify-tools,
  emscripten,
  git,
  nickel,
  tree-sitter,
  diffutils,
  gnused,
  nixdoc,
  jq,
  nushell,
  gh,
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

  changelog = writeTextFile {
    name = "changelog";
    destination = "/bin/changelog";
    executable = true;
    text = ''
      #!${nushell}/bin/nu
      ${readFile ../../bin/changelog.nu}
    '';
  };

in
{
  inherit
    changelog
    generate-nix-documentation
    playground
    verify-documented-usage
    ;
}
