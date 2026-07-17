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

  normaliseGrammar = name: g:
    assert !(g.package != null && g.source.git != null) || throw "topiary: language `${name}` cannot specify both `grammar.package` and `grammar.source.git`";
    {
    source =
      if g.package != null then
        { path = "${g.package}/parser"; }
      else if g.source.git != null then
        { git = g.source.git; }
      else
        throw "topiary: language `${name}` needs `grammar.package` or `grammar.source.git`";
  } // optionalAttrs (g.symbol != null) { inherit (g) symbol; };

  normaliseLanguage = name: lang: {
    inherit (lang) extensions;
    grammar = normaliseGrammar name lang.grammar;
  } // optionalAttrs (lang.indent != null) { inherit (lang) indent; };

  defaultLanguages = (fromNickelFile ../../topiary-config/languages.ncl).languages;
  userLanguages = mapAttrs normaliseLanguage cfg.languages;
  mergedLanguages = (optionalAttrs cfg.includeDefaultLanguages defaultLanguages) // userLanguages;

  configFile = generateNcl {
    name = "languages.ncl";
    config = prefetchLanguages { languages = mergedLanguages; };
    withDefaults = false;
  };

  customQueries = filterAttrs (_: l: l.query.formatting != null) cfg.languages;
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
