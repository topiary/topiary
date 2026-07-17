## Option definitions for declaratively configuring Topiary.
##
## This is the schema half of the PoC for the Home Manager module proposed in
## https://github.com/topiary/bud/discussions/5#discussioncomment-16902980 .
## It only declares `options`; turning the evaluated config into a wrapped
## package is done by the builder in `nix/packages/topiary.nix`, which feeds
## the result of `lib.evalModules` into `generateNcl` + `makeWrapper`.

{ lib, ... }:

let
  inherit (lib) mkOption mkEnableOption types;
in
{
  options.programs.topiary = {
    enable = mkEnableOption "the Topiary formatter";

    package = mkOption {
      type = types.package;
      description = "The Topiary CLI package to wrap.";
    };

    includeDefaultLanguages = mkOption {
      type = types.bool;
      default = true;
      description = ''
        Whether to include Topiary's built-in language definitions in addition
        to the ones declared in {option}`programs.topiary.languages`.
      '';
    };

    languages = mkOption {
      default = { };
      description = "Per-language Topiary configuration, keyed by language name.";
      type = types.attrsOf (
        types.submodule {
          options = {
            extensions = mkOption {
              type = types.listOf types.str;
              description = "File extensions mapped to this language.";
            };

            indent = mkOption {
              type = types.nullOr types.str;
              default = null;
              description = "Indentation string for this language (defaults to Topiary's).";
            };

            grammar = {
              symbol = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = ''
                  Symbol of the language in the compiled grammar, when it differs
                  from `tree_sitter_<name>`.
                '';
              };

              package = mkOption {
                type = types.nullOr types.package;
                default = null;
                description = ''
                  A pre-built tree-sitter grammar derivation providing a `parser`
                  (e.g. `pkgs.tree-sitter-grammars.tree-sitter-foo`). Mutually
                  exclusive with {option}`grammar.source.git`.
                '';
              };

              source.git = mkOption {
                default = null;
                description = "A git source for the tree-sitter grammar, built by Nix.";
                type = types.nullOr (
                  types.submodule {
                    options = {
                      git = mkOption {
                        type = types.str;
                        description = "URL of the git repository.";
                      };
                      rev = mkOption {
                        type = types.str;
                        description = "Revision (commit/tag) to check out.";
                      };
                      nixHash = mkOption {
                        type = types.str;
                        description = "Fixed-output hash of the fetched source.";
                      };
                      subdir = mkOption {
                        type = types.nullOr types.str;
                        default = null;
                        description = "Sub-directory within the repository holding the grammar.";
                      };
                    };
                  }
                );
              };
            };

            query.formatting = mkOption {
              type = types.nullOr types.path;
              default = null;
              description = ''
                Path to a `formatting.scm` query file for this language. When set,
                it is installed into the wrapper's `TOPIARY_LANGUAGE_DIR`.
              '';
            };
          };
        }
      );
    };
  };
}
