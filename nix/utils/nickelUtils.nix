{
  lib,
  nickel,
  runCommand,
  writeText,
}:

let
  inherit (builtins)
    readFile
    toJSON
    fromJSON
    baseNameOf
    ;
  inherit (lib.strings)
    removeSuffix
    ;

  /**
    Transforms a JSON-able Nix value into a JSON file. This file can be consumed
    by Nickel directly.

    # Type

    ```
    toJSONFile : Any -> File
    ```
  */
  toJSONFile = name: e: writeText name (toJSON e);

  /**
    Converts a JSON-able Nickel file into a Nix value.

    # Type

    ```
    fromNickelFile : File -> Any
    ```
  */
  fromNickelFile =
    path:
    let
      jsonDrv = runCommand "${removeSuffix ".ncl" (baseNameOf path)}.json" { } ''
        ${nickel}/bin/nickel export ${path} > $out
      '';
    in
    fromJSON (readFile "${jsonDrv}");

in
{
  inherit
    fromNickelFile
    toJSONFile
    ;
}
