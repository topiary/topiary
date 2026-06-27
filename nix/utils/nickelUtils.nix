{
  lib,
  nickel,
  runCommand,
  writeText,
}:

let
  inherit (builtins)
    concatStringsSep
    isList
    isString
    isAttrs
    readFile
    toJSON
    fromJSON
    baseNameOf
    ;
  inherit (lib.strings)
    removeSuffix
    ;
  inherit (lib.attrsets)
    mapAttrsToList
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

  /**
    Converts a Nix value to Nickel source syntax.

    Strings are JSON-escaped, lists use `[...]`, and attribute sets use
    `{ key = value, ... }` (Nickel record syntax).

    # Type

    ```
    toNickelValue : Any -> String
    ```
  */
  toNickelValue =
    val:
    if isList val then
      "[${concatStringsSep ", " (map toNickelValue val)}]"
    else if isString val then
      toJSON val
    else if isAttrs val then
      "{ ${concatStringsSep ", " (mapAttrsToList (k: v: "${k} = ${toNickelValue v}") val)} }"
    else
      toString val;

in
{
  inherit
    fromNickelFile
    toJSONFile
    toNickelValue
    ;
}
