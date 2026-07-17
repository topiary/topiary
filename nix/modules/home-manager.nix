{ topiaryNix }:

{
  config,
  lib,
  pkgs,
  ...
}:

let
  inherit (lib) mkDefault mkIf optionalAttrs;
  inherit (lib.attrsets) filterAttrs mapAttrs mapAttrsToList mapAttrs' nameValuePair;

  system = pkgs.stdenv.hostPlatform.system;
  topiary = topiaryNix.${system};

  inherit (topiary.lib) generateNcl prefetchLanguages;

  cfg = config.programs.topiary;

  normaliseGrammar =
    name: g:
    {
      source =
        if g.package != null then
          { path = "${g.package}/parser"; }
        else if g.source.git != null then
          {
            git = {
              git = g.source.git.url;
              inherit (g.source.git) rev;
              nixHash = g.source.git.hash;
            }
            // optionalAttrs (g.source.git.subdir != null) { inherit (g.source.git) subdir; };
          }
        else
          throw "topiary: language `${name}` needs `grammar.package` or `grammar.source.git`";
    }
    // optionalAttrs (g.symbol != null) { inherit (g) symbol; };

  normaliseLanguage =
    name: lang:
    {
      inherit (lang) extensions;
      grammar = normaliseGrammar name lang.grammar;
    }
    // optionalAttrs (lang.indent != null) { inherit (lang) indent; };

  defaultLanguages = topiary.lib.defaultConfig.languages;
  userLanguages = mapAttrs normaliseLanguage cfg.languages;
  mergedLanguages = (optionalAttrs cfg.includeDefaultLanguages defaultLanguages) // userLanguages;

  configFile = generateNcl {
    name = "languages.ncl";
    config = prefetchLanguages { languages = mergedLanguages; };
    withDefaults = false;
  };

  customQueries = filterAttrs (_: l: l.query.formatting != null) cfg.languages;
  queryFiles = mapAttrs' (
    name: l:
    nameValuePair "topiary/queries/${name}/formatting.scm" { source = l.query.formatting; }
  ) customQueries;
in
{
  imports = [ ./topiary.nix ];

  config = mkIf cfg.enable {
    programs.topiary.package = mkDefault topiary.packages.topiary-cli;

    assertions = mapAttrsToList (name: lang: {
      assertion = !(lang.grammar.package != null && lang.grammar.source.git != null);
      message = "topiary: language `${name}` cannot specify both `grammar.package` and `grammar.source.git`";
    }) cfg.languages;

    home.packages = [ cfg.package ];

    xdg.configFile =
      {
        "topiary/languages.ncl".source = configFile;
      }
      // queryFiles;
  };
}
