{
  pkgs,
  advisory-db,
  craneLib,
  prefetchLanguagesFile,
  prefetchLanguagesNickelFile,

  cargoProfile ? "release",
  runBenchmark ? true,
}:

let
  inherit (pkgs.lib)
    fileset
    optional
    optionals
    makeOverridable
    ;

  # Inject `CARGO_PROFILE` into every derivation built on top of `mkCargoDerivation`
  craneLib' = craneLib.overrideScope (
    _final: prev: {
      mkCargoDerivation = args: prev.mkCargoDerivation ({ CARGO_PROFILE = cargoProfile; } // args);
      stdenvSelector =
        p: if p.stdenv.hostPlatform.isLinux then p.stdenvAdapters.useMoldLinker p.stdenv else p.stdenv;
    }
  );

  commonArgs = {
    pname = "topiary";

    src = fileset.toSource {
      root = ../..;
      fileset = fileset.unions [
        ../.
        ../../.cargo
        ../../Cargo.lock
        ../../Cargo.toml
        ../../examples
        ../../languages.ncl
        ../../topiary-cli
        ../../topiary-config
        ../../topiary-core
        ../../topiary-queries
        ../../topiary-tree-sitter-facade
        ../../topiary-web-tree-sitter-sys
      ];
    };

    nativeBuildInputs =
      with pkgs;
      [
        binaryen
        wasm-bindgen-cli
        pkg-config
      ]
      ++ optionals stdenv.hostPlatform.isDarwin [
        libiconv
      ];

    buildInputs = with pkgs; [
      openssl.dev
    ];
  };

  prepareTopiaryDefaultConfiguration = ''
    cp ${prefetchLanguagesNickelFile ../../topiary-config/languages.ncl} topiary-config/languages.ncl
  '';

  cargoArtifacts = craneLib'.buildDepsOnly commonArgs;

  workspace-tests = craneLib'.cargoNextest (
    commonArgs
    // {
      inherit cargoArtifacts;
      CARGO_PROFILE = "dev";
      pname = "topiary-workspace-tests";
      cargoExtraArgs = "--locked";
      cargoNextestExtraArgs = "--workspace --all-features";
      preConfigurePhases = [ "prepareTopiaryDefaultConfiguration" ];
      inherit prepareTopiaryDefaultConfiguration;
    }
  );

  # `cargo nextest` does not run doctests
  workspace-doctests = craneLib'.cargoDocTest (
    commonArgs
    // {
      inherit cargoArtifacts;
      pname = "topiary-workspace-doctests";
      cargoExtraArgs = "--workspace --all-features --locked";
      preConfigurePhases = [ "prepareTopiaryDefaultConfiguration" ];
      inherit prepareTopiaryDefaultConfiguration;
    }
  );

  clippy = craneLib'.cargoClippy (
    commonArgs
    // {
      inherit cargoArtifacts;

      cargoExtraArgs = "--workspace --all-features --locked";
      cargoClippyExtraArgs = "--all-targets -- --deny warnings";
    }
  );

  fmt = craneLib'.cargoFmt commonArgs;

  audit = craneLib'.cargoAudit (
    commonArgs
    // {
      inherit advisory-db;

      # A minimal source, rather than commonArgs' src, so that
      # cargo-audit sees its configuration without audit.toml changes
      # invalidating the sources of every other derivation.
      src = fileset.toSource {
        root = ../..;
        fileset = fileset.unions [
          ../../Cargo.lock
          ../../.cargo/audit.toml
        ];
      };
    }
  );

  benchmark =
    if runBenchmark then
      craneLib'.buildPackage (
        commonArgs
        // {
          inherit cargoArtifacts;
          cargoTestCommand = "cargo bench --profile ${cargoProfile}";
          preConfigurePhases = [ "prepareTopiaryDefaultConfiguration" ];
          inherit prepareTopiaryDefaultConfiguration;
        }
      )
    else
      pkgs.runCommand "topiary-benchmark-skipped" { } "touch $out";

  client-app = craneLib'.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
      pname = "client-app";
      cargoExtraArgs = "-p client-app";
    }
  );

  topiary-core = craneLib'.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
      pname = "topiary-core";
      cargoExtraArgs = "-p topiary-core";
      preConfigurePhases = [ "prepareTopiaryDefaultConfiguration" ];
      inherit prepareTopiaryDefaultConfiguration;
    }
  );

  topiary-cli = makeOverridable (
    {
      prefetchGrammars ? false,
    }:
    craneLib'.buildPackage (
      commonArgs
      // {
        inherit cargoArtifacts;
        pname = "topiary";
        cargoExtraArgs = "-p topiary-cli";

        checkPhaseCargoCommand = "cargo nextest run -p topiary-cli --no-default-features --cargo-profile ${cargoProfile}";
        nativeBuildInputs = commonArgs.nativeBuildInputs ++ [ pkgs.cargo-nextest ];

        preConfigurePhases = optional prefetchGrammars "prepareTopiaryDefaultConfiguration";
        inherit prepareTopiaryDefaultConfiguration;

        postInstall = ''
          mkdir -p $out/share/queries
          cp -r topiary-queries/queries/* $out/share/queries
        '';

        # Set TOPIARY_LANGUAGE_DIR to the Nix store
        # for the build
        TOPIARY_LANGUAGE_DIR = "${placeholder "out"}/share/queries";

        # Set TOPIARY_LANGUAGE_DIR to the working directory
        # in a development shell
        shellHook = ''
          export TOPIARY_LANGUAGE_DIR=$PWD/queries
        '';

        meta.mainProgram = "topiary";
      }
    )
  ) { };

  topiary-queries = craneLib'.buildPackage (
    commonArgs
    // {
      pname = "topiary-queries";
      cargoExtraArgs = "-p topiary-queries";
      postInstall = ''
        mkdir -p $out/share/queries
        cp -r --no-preserve=mode topiary-queries/queries/* $out/share/queries
      '';
    }
  );

  # We need to pin to mdBook v0.4 for the time being; v0.5 introduces
  # breaking changes
  mdbook =
    let
      src = pkgs.fetchCrate {
        pname = "mdbook";
        version = "0.4.47";
        hash = "sha256-ReayYpD6GIsc7B3+ekCU37tN4+knhei8P0BsJOZyz/U=";
      };
      cargoArtifacts = craneLib'.buildDepsOnly {
        inherit src;
        pname = "mdbook";
        version = "0.4.47";
      };
    in
    craneLib'.buildPackage {
      inherit src cargoArtifacts;
      pname = "mdbook";
      version = "0.4.47";

      # Tests require the guide directory which isn't included in the crate
      doCheck = false;

      meta = {
        description = "Creates a book from markdown files";
        mainProgram = "mdbook";
      };
    };

  topiary-book = pkgs.stdenv.mkDerivation {
    pname = "topiary-book";
    version = "1.0";

    src = fileset.toSource {
      root = ../..;
      fileset = fileset.unions [
        ../../docs/book
        ../.
      ];
    };

    nativeBuildInputs = [ mdbook ];

    buildPhase = ''
      cd docs/book
      mdbook build
    '';

    installPhase = ''
      mkdir -p $out
      cp -r book/* $out
    '';
  };

  mdbook-manmunge =
    let
      src = pkgs.fetchCrate {
        pname = "mdbook-manmunge";
        version = "0.0.1";
        hash = "sha256-mrZTzzk9X71NC/nJME+FbQYM+epin5sByFA0RVhcvRw=";
      };
      cargoArtifacts = craneLib'.buildDepsOnly {
        inherit src;
        pname = "mdbook-manmunge";
        version = "0.0.1";
      };
    in
    craneLib'.buildPackage {
      inherit src cargoArtifacts;
      pname = "mdbook-manmunge";
      version = "0.0.1";

      meta = {
        description = "mdBook pre- and post-processor to help munge (a subset of) the Topiary Book into manpages with mdbook-man";
        mainProgram = "mdbook-manmunge";
      };
    };

  topiary-manpages = pkgs.stdenv.mkDerivation {
    pname = "topiary-manpages";
    version = "1.0";

    src = fileset.toSource {
      root = ../..;
      fileset = fileset.unions [
        ../../docs/manpages
        ../../docs/book/src/cli
      ];
    };

    nativeBuildInputs = [
      pkgs.gzip
      mdbook
      pkgs.mdbook-man
      mdbook-manmunge
    ];

    buildPhase = ''
      cd docs/manpages
      make all
    '';

    installPhase = ''
      MAN_DIR=$out/share/man \
      make install
    '';

    meta = {
      description = "Topiary manpages";
    };
  };

  topiary-docker =
    let
      cli = topiary-cli.override { prefetchGrammars = true; };
    in
    pkgs.dockerTools.buildLayeredImage {
      name = "topiary";
      tag = "latest";

      contents = [
        cli
        pkgs.dockerTools.caCertificates
      ];

      config = {
        Entrypoint = [ "${cli}/bin/topiary" ];
        Labels = {
          "org.opencontainers.image.source" = "https://github.com/topiary/topiary";
          "org.opencontainers.image.description" = "A general code formatter based on Tree-sitter";
        };
      };
    };

in
{
  inherit
    # passthru
    clippy
    fmt
    audit
    benchmark
    workspace-tests
    workspace-doctests
    client-app
    topiary-core
    topiary-cli
    topiary-docker
    topiary-queries
    mdbook
    mdbook-manmunge
    topiary-book
    topiary-manpages
    ;

  default = topiary-cli;
}
