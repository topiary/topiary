{
  pkgs,
  advisory-db,
  craneLib,
  prefetchLanguagesFile,
  prefetchLanguagesNickelFile,
  prefetchLanguages,
  generateNcl,
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
      inherit (pkgs.lib) optionalAttrs mapAttrs filterAttrs mkDefault;
      inherit (pkgs.lib.strings) optionalString concatStringsSep;
      inherit (pkgs.lib.attrsets) mapAttrsToList;

      eval = pkgs.lib.evalModules {
        specialArgs = { inherit pkgs; };
        modules = [
          ../modules/topiary.nix
          { programs.topiary.package = mkDefault topiary-cli; }
          settings
        ];
      };
      cfg = eval.config.programs.topiary;

      # Normalise a grammar block (proposal schema) into the internal source
      # shape understood by `prefetchLanguages` (`{ git = {...} }` or
      # `{ path = ...; }`), which then resolves both to a store path.
      normaliseGrammar =
        name: g:
        assert !(g.package != null && g.source.git != null) || throw "topiary: language `${name}` cannot specify both `grammar.package` and `grammar.source.git`";
        {
          source =
            if g.package != null then
              { path = "${g.package}/parser"; }
            else if g.source.git != null then
              {
                git = {
                  git = g.source.git.url;
                  inherit (g.source.git) rev;
                  nixHash = g.source.git.hash;
                }
                // optionalAttrs (g.source.git.subdir != null) { inherit (g.source.git) subdir; };
              }
            else
              throw "topiary: language `${name}` needs `grammar.package` or `grammar.source.git`";
        }
        // optionalAttrs (g.symbol != null) { inherit (g) symbol; };

      # Drop the `query` field (handled via TOPIARY_LANGUAGE_DIR) and normalise
      # the grammar into the internal config shape.
      normaliseLanguage =
        name: lang:
        {
          inherit (lang) extensions;
          grammar = normaliseGrammar name lang.grammar;
        }
        // optionalAttrs (lang.indent != null) { inherit (lang) indent; };

      defaultLanguages = (import ../languages.nix).languages;
      userLanguages = mapAttrs normaliseLanguage cfg.languages;
      mergedLanguages = (optionalAttrs cfg.includeDefaultLanguages defaultLanguages) // userLanguages;

      configFile = generateNcl {
        name = "languages.ncl";
        config = prefetchLanguages { languages = mergedLanguages; };
        withDefaults = false;
      };

      # Languages carrying a custom `query.formatting` file.
      customQueries = filterAttrs (_: l: l.query.formatting != null) cfg.languages;
      hasCustomQueries = customQueries != { };

      # A query directory laid out as `<lang>/formatting.scm`, optionally seeded
      # with the package's bundled queries so the defaults keep working.
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
    in
    pkgs.stdenv.mkDerivation {
      pname = "topiary-with-nix-config";
      inherit (cfg.package) version;

      dontUnpack = true;

      nativeBuildInputs = [ pkgs.makeWrapper ];

      installPhase = ''
        runHook preInstall

        mkdir -p $out/bin
        makeWrapper ${cfg.package}/bin/topiary $out/bin/topiary \
          --set-default TOPIARY_CONFIG_FILE ${configFile} ${
            optionalString hasCustomQueries "--set-default TOPIARY_LANGUAGE_DIR ${queriesDir}"
          }

        runHook postInstall
      '';

      passthru = {
        inherit configFile queriesDir;
        config = cfg;
      };

      meta = cfg.package.meta or { } // {
        mainProgram = "topiary";
      };
    };

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

  inherit mkTopiaryWithNixConfig;

  default = topiary-cli;
}
