## Option definitions for declaratively configuring Topiary.
##
## This is the schema half of the PoC for the Home Manager module proposed in
## https://github.com/topiary/bud/discussions/5#discussioncomment-16902980 .
## It only declares `options`; turning the evaluated config into a wrapped
## package is done by the builder in `nix/packages/topiary.nix`, which feeds
## the result of `lib.evalModules` into `generateNcl` + `makeWrapper`.

{ lib, pkgs, ... }:

let
  inherit (lib) mkOption mkEnableOption types;
  jsonFormat = pkgs.formats.json { };
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
        to the ones declared in {option}`programs.topiary.settings.languages`.
      '';
    };

    settings = mkOption {
      default = { };
      description = "Topiary configuration, evaluated into languages.ncl.";
      type = types.submodule {
        freeformType = jsonFormat.type;
        options = {
          languages = mkOption {
            default = { };
            description = "Per-language Topiary configuration, keyed by language name.";
            type = types.attrsOf (
              types.submodule {
                freeformType = jsonFormat.type;
                options = {
                  grammar = {
                    package = mkOption {
                      type = types.nullOr types.package;
                      default = null;
                      description = ''
                        A pre-built tree-sitter grammar derivation providing a `parser`
                        (e.g. `pkgs.tree-sitter-grammars.tree-sitter-foo`). Mutually
                        exclusive with `grammar.source.git`.
                      '';
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
      };
    };
  };
}
