{
  pkgs,
  lib,
  generateNcl,
  prefetchLanguages,
  fromNickelFile,
}:

{ cfg }:

let
  inherit (lib) optionalAttrs;
  inherit (lib.attrsets) mapAttrs filterAttrs mapAttrs' nameValuePair mapAttrsToList;
  inherit (lib.strings) optionalString concatStringsSep;

  cleanGrammar = g: builtins.removeAttrs g [ "package" ];
  cleanLanguage = lang: builtins.removeAttrs lang [ "query" ];

  normaliseGrammar = name: g:
    let
      hasPackage = g.package != null;
      hasGit = g.source.git or null != null;
      hasPath = g.source.path or null != null;
    in
    assert !(hasPackage && hasGit) || throw "topiary: language `${name}` cannot specify both `grammar.package` and `grammar.source.git`";
    cleanGrammar (g // {
      source =
        if hasPackage then
          { path = "${g.package}/parser"; }
        else if hasGit then
          { git = g.source.git; }
        else if hasPath then
          { path = g.source.path; }
        else
          throw "topiary: language `${name}` needs `grammar.package`, `grammar.source.git`, or `grammar.source.path`";
    });

  normaliseLanguage = name: lang: cleanLanguage (lang // {
    grammar = normaliseGrammar name lang.grammar;
  });

  defaultLanguages = (fromNickelFile ../../topiary-config/languages.ncl).languages;
  userLanguages = mapAttrs normaliseLanguage (cfg.settings.languages or { });
  mergedLanguages = (optionalAttrs cfg.includeDefaultLanguages defaultLanguages) // userLanguages;

  configFile = generateNcl {
    name = "languages.ncl";
    config = prefetchLanguages (cfg.settings // { languages = mergedLanguages; });
    withDefaults = false;
  };

  customQueries = filterAttrs (_: l: l.query.formatting or null != null) (cfg.settings.languages or { });
  hasCustomQueries = customQueries != { };

  queryFiles = mapAttrs' (
    name: l: nameValuePair "topiary/queries/${name}/formatting.scm" { source = l.query.formatting; }
  ) customQueries;

  queriesDir = pkgs.runCommand "topiary-queries" { } ''
    mkdir -p $out
    ${optionalString cfg.includeDefaultLanguages ''
      cp -r --no-preserve=mode ${cfg.package}/share/queries/. $out/
    ''}
    ${concatStringsSep "\n" (
      mapAttrsToList (name: l: ''
        mkdir -p $out/${name}
        cp ${l.query.formatting} $out/${name}/formatting.scm
      '') customQueries
    )}
  '';

in {
  inherit configFile queryFiles queriesDir hasCustomQueries;
}
