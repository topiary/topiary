{
  pkgs,
  advisory-db,
  craneLib,
  prefetchLanguagesFile,
  prefetchLanguagesNickelFile,
  prefetchLanguages,
  generateNcl,
  fromNickelFile,
  topiaryLib,
}:

let
  inherit (pkgs.lib)
    fileset
    optional
    optionals
    makeOverridable
    ;

  commonArgs = {
    pname = "topiary";

    src = fileset.toSource {
      root = ../..;
      fileset = fileset.unions [
        ../../Cargo.lock
        ../../Cargo.toml
        ../../languages.ncl
        ../../examples
        ../../topiary-core
        ../../topiary-cli
        ../../topiary-config
        ../../topiary-queries
        ../../topiary-tree-sitter-facade
        ../../topiary-web-tree-sitter-sys
        ../.
      ];
    };

    nativeBuildInputs =
      with pkgs;
      [
        binaryen
        wasm-bindgen-cli
        pkg-config
      ]
      ++ optionals stdenv.isDarwin [
        libiconv
      ];

    buildInputs = with pkgs; [
      openssl.dev
    ];
  };

  prepareTopiaryDefaultConfiguration = ''
    cp ${prefetchLanguagesNickelFile ../../topiary-config/languages.ncl} topiary-config/languages.ncl
  '';

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  clippy = craneLib.cargoClippy (
    commonArgs
    // {
      inherit cargoArtifacts;
      cargoClippyExtraArgs = "-- --deny warnings";
    }
  );

  fmt = craneLib.cargoFmt commonArgs;

  audit = craneLib.cargoAudit (
    commonArgs
    // {
      inherit advisory-db;
    }
  );

  benchmark = craneLib.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
      cargoTestCommand = "cargo bench --profile release";
      preConfigurePhases = [ "prepareTopiaryDefaultConfiguration" ];
      inherit prepareTopiaryDefaultConfiguration;
    }
  );

  client-app = craneLib.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
      pname = "client-app";
      cargoExtraArgs = "-p client-app";
    }
  );

  topiary-core = craneLib.buildPackage (
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
    craneLib.buildPackage (
      commonArgs
      // {
        inherit cargoArtifacts;
        pname = "topiary";
        cargoExtraArgs = "-p topiary-cli";
        cargoTestExtraArgs = "--no-default-features";

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

  topiary-queries = craneLib.buildPackage (
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
      cargoArtifacts = craneLib.buildDepsOnly {
        inherit src;
        pname = "mdbook";
        version = "0.4.47";
      };
    in
    craneLib.buildPackage {
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
      cargoArtifacts = craneLib.buildDepsOnly {
        inherit src;
        pname = "mdbook-manmunge";
        version = "0.0.1";
      };
    in
    craneLib.buildPackage {
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

  # This runs the Topiary CLI in a controlled PTY for stable output
  # while testing in CI (90 columns and no ANSI extensions)
  topiary-wrapped = pkgs.writeShellApplication {
    name = "topiary-wrapped";

    runtimeInputs = [
      topiary-cli
      pkgs.expect
    ];

    text = ''
      export COLUMNS=90
      export NO_COLOR=1

      unbuffer topiary "$@"
    '';
  };

  # Topiary CLI wrapped with a declaratively-configured `languages.ncl`.
  #
  # This is a PoC for the Home Manager module proposed in
  # https://github.com/topiary/bud/discussions/5#discussioncomment-16902980 .
  # The option schema lives in `nix/modules/topiary.nix`; here we evaluate it
  # with `lib.evalModules`, then turn the resulting config into a wrapped
  # package (generated `languages.ncl` with grammar sources normalised to
  # store paths, plus a `TOPIARY_LANGUAGE_DIR` for any custom queries).
  #
  #   mkTopiaryWithNixConfig {
  #     programs.topiary = {
  #       includeDefaultLanguages = true;
  #       languages.foo = {
  #         extensions = [ "foo" ];
  #         indent = "  ";                          # optional
  #         grammar = {
  #           symbol = "tree_sitter_foo";           # optional
  #           package = pkgs.tree-sitter-grammars.tree-sitter-foo;  # OR
  #           source.git = { url = "..."; rev = "..."; hash = "sha256-..."; };
  #         };
  #         query.formatting = ./foo.scm;           # optional
  #       };
  #     };
  #   }
  mkTopiaryWithNixConfig =
    settings:
    let
      inherit (pkgs.lib) mkDefault;

      eval = pkgs.lib.evalModules {
        specialArgs = { inherit pkgs; };
        modules = [
          ../modules/topiary.nix
          { programs.topiary.package = mkDefault topiary-cli; }
          settings
        ];
      };
      cfg = eval.config.programs.topiary;

      configInfo = topiaryLib.evalConfig { inherit cfg; };
      
      wrapped = topiaryLib.wrapWithConfigFile {
        package = cfg.package;
        inherit (configInfo) configFile;
        languageDir = if configInfo.hasCustomQueries then configInfo.queriesDir else null;
      };
    in
    wrapped.overrideAttrs (old: {
      name = "topiary-with-nix-config-${cfg.package.version}";
      pname = "topiary-with-nix-config";
      inherit (cfg.package) version;
      passthru = (old.passthru or {}) // {
        inherit (configInfo) configFile queriesDir;
        config = cfg;
      };
      meta = cfg.package.meta or { } // {
        mainProgram = "topiary";
      };
    });

  # Instance built from the declarative settings in `nix/topiary-config.nix`,
  # evaluated through the option schema with `lib.evalModules`.
  topiary-with-nix-config = mkTopiaryWithNixConfig ../topiary-config.nix;

in
{
  inherit
    # passthru
    clippy
    fmt
    audit
    benchmark
    client-app
    topiary-core
    topiary-cli
    topiary-docker
    topiary-queries
    mdbook
    mdbook-manmunge
    topiary-book
    topiary-manpages
    topiary-wrapped
    topiary-with-nix-config
    ;

  default = topiary-cli;
}
