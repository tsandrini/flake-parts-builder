# --- flake.nix
{
  description = "Nix flakes interactive template builder based on flake-parts written in Rust.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    let
      inherit (inputs.nixpkgs) lib;

      tsandrini = {
        email = "tomas.sandrini@seznam.cz";
        name = "Tomáš Sandrini";
        github = "tsandrini";
        githubId = 21975189;
      };
    in
    flake-parts.lib.mkFlake { inherit inputs; } {

      systems = import inputs.systems;

      flake.lib = rec {
        flatten = attrs: lib.collect (x: !lib.isAttrs x) attrs;

        mapFilterAttrs =
          pred: f: attrs:
          lib.filterAttrs pred (lib.mapAttrs' f attrs);

        mapModules =
          dir: fn:
          mapFilterAttrs (n: v: v != null && !(lib.hasPrefix "_" n) && !(lib.lib.hasPrefix ".git" n)) (
            n: v:
            let
              path = "${toString dir}/${n}";
            in
            if v == "directory" && builtins.pathExists "${path}/default.nix" then
              lib.nameValuePair n (fn path)
            else if v == "directory" then
              lib.nameValuePair n (mapModules path fn)
            else if v == "regular" && n != "default.nix" && lib.hasSuffix ".nix" n then
              lib.nameValuePair (lib.removeSuffix ".nix" n) (fn path)
            else
              lib.nameValuePair "" null
          ) (builtins.readDir dir);

        # NOTE In case anyone ditches _bootstrap and wants to use
        # load-parts directly from here.
        loadParts = dir: flatten (mapModules dir (x: x));
      };

      perSystem =
        {
          config,
          pkgs,
          system,
          ...
        }:
        {
          packages = rec {
            default = builder;

            builder =
              let
                package =
                  {
                    lib,
                    rustPlatform,
                    nixfmt-rfc-style,
                    tsandrini,
                  }:
                  rustPlatform.buildRustPackage {
                    name = "flake-parts-builder";
                    version = "1.0.0";

                    src = [
                      ./src
                      ./Cargo.toml
                      ./Cargo.lock
                    ];

                    unpackPhase = ''
                      runHook preUnpack
                      for srcItem in $src; do
                        if [ -d "$srcItem" ]; then
                          cp -r "$srcItem" $(stripHash "$srcItem")
                        else
                          cp "$srcItem" $(stripHash "$srcItem")
                        fi
                      done
                      runHook postUnpack
                    '';

                    cargoSha256 = "sha256-yWViuPayJX1WFZbhMQPAQJoRUDJohF7NHGMe/y84TdI=";

                    buildInputs = [ nixfmt-rfc-style ];

                    meta = with lib; {
                      homepage = "https://github.com/tsandrini/flake-parts-builder";
                      description = "Nix flakes interactive template builder based on flake-parts written in Rust.";
                      license = licenses.mit;
                      platforms = [ system ];
                      maintainers = [ tsandrini ];
                      mainProgram = "flake-parts-builder";
                    };
                  };
              in
              pkgs.callPackage package { inherit tsandrini; };

            flake-parts =
              let
                package =
                  {
                    lib,
                    stdenv,
                    tsandrini,
                  }:
                  stdenv.mkDerivation {
                    name = "flake-parts";
                    version = "1.0.0";
                    src = ./flake-parts;

                    dontConfigure = true;
                    dontBuild = true;
                    dontCheck = true;

                    installPhase = ''
                      mkdir -p $out/flake-parts
                      cp -rv $src/* $out/flake-parts
                    '';

                    meta = with lib; {
                      homepage = "https://github.com/tsandrini/flake-parts-builder";
                      description = "The base collection of flake-parts for the flake-parts-builder.";
                      license = licenses.mit;
                      platforms = [ system ];
                      maintainers = [ tsandrini ];
                    };
                  };
              in
              pkgs.callPackage package { inherit tsandrini; };
          };

          devShells = rec {
            default = dev;

            dev =
              let
                package =
                  {
                    mkShell,
                    nil,
                    statix,
                    deadnix,
                    nix-output-monitor,
                    nixfmt-rfc-style,
                    commitizen,
                    cz-cli,
                    gh,
                    gh-dash,
                    markdownlint-cli,
                  }:
                  mkShell {
                    buildInputs = [
                      # -- NIX UTILS --
                      nil # Yet another language server for Nix
                      statix # Lints and suggestions for the nix programming language
                      deadnix # Find and remove unused code in .nix source files
                      nix-output-monitor # Processes output of Nix commands to show helpful and pretty information
                      nixfmt-rfc-style # An opinionated formatter for Nix

                      # -- GIT RELATED UTILS --
                      commitizen # Tool to create committing rules for projects, auto bump versions, and generate changelogs
                      cz-cli # The commitizen command line utility
                      # fh # The official FlakeHub CLI
                      gh # GitHub CLI tool
                      gh-dash # Github Cli extension to display a dashboard with pull requests and issues

                      # -- BASE LANG UTILS --
                      markdownlint-cli # Command line interface for MarkdownLint
                      # nodePackages.prettier # Prettier is an opinionated code formatter
                      # typos # Source code spell checker

                      # -- (YOUR) EXTRA PKGS --
                    ];

                    shellHook = ''
                      # Welcome splash text
                      echo ""; echo -e "\e[1;37;42mWelcome to the flake-parts-builder devshell!\e[0m"; echo ""
                    '';
                  };
              in
              pkgs.callPackage package { };
          };
        };
    };
}
