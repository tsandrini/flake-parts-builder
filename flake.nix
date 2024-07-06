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

            builder = pkgs.rustPlatform.buildRustPackage {
              name = "flake-parts-builder";
              version = "1.0.0";

              src = builtins.path {
                path = ./.;
                filter = path: type: !(builtins.elem (/. + path) [ ./flake-parts ]);
              };

              cargoSha256 = "sha256-6rVpTWcGX+sNCEq14AEkqC8Ui+tnso50ZXMv28evMxg=";

              buildInputs = with pkgs; [ nixfmt-rfc-style ];
            };

            flake-parts = pkgs.stdenv.mkDerivation {
              name = "flake-parts";
              version = "1.0.0";
              src = ./flake-parts;

              dontConfigure = true;
              dontBuild = true;

              installPhase = ''
                mkdir -p $out/flake-parts
                cp -rv $src/* $out/flake-parts
              '';

              meta = with pkgs.lib; {
                homepage = "TODO";
                description = "TODO";
                license = licenses.mit;
                platforms = [ system ];
                maintainers = [ ];
              };
            };
          };

          devShells = rec {
            default = dev;

            dev = pkgs.mkShell {
              buildInputs = with pkgs; [
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
          };
        };
    };
}
