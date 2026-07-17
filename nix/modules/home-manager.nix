{ topiaryNix }:

{
  config,
  lib,
  pkgs,
  ...
}:

let
  inherit (lib) mkDefault mkIf;
  inherit (lib.attrsets) mapAttrsToList;

  system = pkgs.stdenv.hostPlatform.system;
  topiary = topiaryNix.${system};

  cfg = config.programs.topiary;

  configInfo = topiary.lib.evalConfig { inherit cfg; };
in
{
  imports = [ ./topiary.nix ];

  config = mkIf cfg.enable {
    programs.topiary.package = mkDefault topiary.packages.topiary-cli;

    assertions = mapAttrsToList (name: lang: {
      assertion = !(lang.grammar.package != null && lang.grammar.source.git or null != null);
      message = "topiary: language `${name}` cannot specify both `grammar.package` and `grammar.source.git`";
    }) (cfg.settings.languages or { });

    home.packages = [ cfg.package ];

    xdg.configFile =
      {
        "topiary/languages.ncl".source = configInfo.configFile;
      }
      // configInfo.queryFiles;
  };
}
