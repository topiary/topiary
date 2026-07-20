{
  pkgs,
  lib,
  generateNcl,
  prefetchLanguages,
  fromNickelFile,
}:

{ cfg }:

let
  inherit (lib) getExe optionalAttrs;
  inherit (lib.attrsets) mapAttrs filterAttrs mapAttrs' nameValuePair mapAttrsToList;
  inherit (lib.strings) optionalString concatStringsSep;

  cleanGrammar = g: builtins.removeAttrs g [ "package" ];
  cleanLanguage = lang: builtins.removeAttrs lang [ "query" ];

  normaliseGrammar = name: g:
    let
      hasPackage = g.package != null;
    in
    cleanGrammar (g // {
      source = (g.source or { }) // (optionalAttrs hasPackage { path = "${g.package}/parser"; });
    });

  normaliseLanguage = name: lang: cleanLanguage (lang // {
    grammar = normaliseGrammar name lang.grammar;
  });

  defaultLanguages = (fromNickelFile ../../topiary-config/languages.ncl).languages;
  userLanguages = mapAttrs normaliseLanguage (cfg.settings.languages or { });
  mergedLanguages = (optionalAttrs cfg.includeDefaultLanguages defaultLanguages) // userLanguages;

  generatedConfigFile = generateNcl {
    name = "languages-unvalidated.ncl";
    config = prefetchLanguages (cfg.settings // { languages = mergedLanguages; });
    withDefaults = false;
  };

  configFile = pkgs.runCommand "languages.ncl" { } ''
    # Loading the configuration is enough to apply its Nickel contracts.
    ${getExe cfg.package} --configuration ${generatedConfigFile} config show-sources > /dev/null
    cp ${generatedConfigFile} $out
  '';

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
