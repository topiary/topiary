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

    home.packages = [ cfg.package ];

    xdg.configFile =
      {
        "topiary/languages.ncl".source = configInfo.configFile;
      }
      // configInfo.queryFiles;
  };
}
