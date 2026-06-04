final: _:

let
  inherit (builtins)
    elemAt
    filter
    fromTOML
    length
    map
    readFile
    ;

  cargoLock = fromTOML (readFile ../../Cargo.lock);
  wasmBindgenCargoVersions = map ({ version, ... }: version) (
    filter ({ name, ... }: name == "wasm-bindgen") cargoLock.package
  );
  wasmBindgenVersion =
    assert length wasmBindgenCargoVersions == 1;
    elemAt wasmBindgenCargoVersions 0;

in
{
  wasm-bindgen-cli = final.buildWasmBindgenCli rec {
    src = final.fetchCrate {
      pname = "wasm-bindgen-cli";
      version = wasmBindgenVersion;
      hash = "sha256-vO4RSxi/sMWxmsEs3GuljdMfIRSu75A+Q+c5wgYToRU=";
    };

    cargoDeps = final.rustPlatform.fetchCargoVendor {
      inherit src;
      pname = "${src.pname}-${src.version}";
      hash = "sha256-Inup6vvJSG5ghNyeDPyZbfZo4d0LsMG2OJfStoaeDBs=";
    };
  };
}
