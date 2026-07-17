## Sample declarative Topiary configuration (PoC settings).
##
## This is a *settings* module: it sets values for the options declared in
## `nix/modules/topiary.nix`. `nix/packages/topiary.nix` feeds it through
## `lib.evalModules` and builds `topiary-with-config-file` from the result.
##
## Edit this file and run `nix run .#topiary-with-config-file -- config` to
## test the schema end-to-end.

{ pkgs, ... }:

{
  programs.topiary = {
    enable = true;

    # Keep Topiary's built-in languages in addition to the ones below.
    includeDefaultLanguages = true;

    # Example of an extra/overriding language. Uncomment and adjust to test.
    #
    # settings.languages.bash = {
    #   extensions = [ "sh" "bash" ];
    #   grammar.source.git = {
    #     git = "https://github.com/tree-sitter/tree-sitter-bash.git";
    #     rev = "d1a1a3fe7189fdab5bd29a54d1df4a5873db5cb1";
    #     nixHash = "sha256-XiiEI7/6b2pCZatO8Z8fBgooKD8Z+SFQJNdR/sGGkgE=";
    #   };
    #   query.formatting = ./../topiary-queries/queries/bash.scm;
    # };
  };
}
