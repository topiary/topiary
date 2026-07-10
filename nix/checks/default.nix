{
  runCommand,
  lib,
  emptyFile,
  topiaryPkgs,
  binPkgs,
  gitHook,
}:

let
  inherit (builtins) deepSeq;

in
{
  inherit (topiaryPkgs)
    clippy
    fmt
    topiary-core
    audit
    benchmark
    topiary-cli
    ;

  # Check that the `lib.gitHook` output builds/evaluates correctly. `deepSeq e1
  # e2` evaluates `e1` strictly in depth before returning `e2`. We use this
  # trick because checks need to be derivations, which `lib.gitHook` is not.
  gitHook = deepSeq gitHook emptyFile;

  verify-documented-usage =
    runCommand "verify-documented-usage"
      {
        nativeBuildInputs = [ binPkgs.verify-documented-usage ];
        TOPIARY = lib.getExe topiaryPkgs.topiary-cli;
      }
      ''
        mkdir -p docs/book/src/cli
        cp -r ${../../docs/book/src/cli/usage} docs/book/src/cli/usage
        verify-documented-usage
        touch $out
      '';
}
