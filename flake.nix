{
  description = "PracticalFlakesTemplate - Highly opinionated nix flakes starter template that focuses on modularity.";

  inputs = {
    # --- BASE DEPENDENCIES ---
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # --- DEV DEPENDENCIES ---
    devenv.url = "github:cachix/devenv";
    devenv-root = {
      url = "file+file:///dev/null";
      flake = false;
    };
    mk-shell-bin.url = "github:rrbutani/nix-mk-shell-bin";
    nix2container = {
      url = "github:nlewo/nix2container";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix.url = "github:numtide/treefmt-nix";

    # --- (YOUR) EXTRA DEPENDENCIES ---
  };

  # NOTE Here you can add additional binary cache substituers that you trust.
  # There are also some sensible default caches commented out that you
  # might consider using.
  nixConfig = {
    extra-substituters = [
      "https://cache.nixos.org"
      "https://nix-community.cachix.org/"
      "https://devenv.cachix.org"
      "https://tsandrini.cachix.org"
    ];
    extra-trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
      "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw="
      "tsandrini.cachix.org-1:t0AzIUglIqwiY+vz/WRWXrOkDZN8TwY3gk+n+UDt4gw="
    ];
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = with inputs; [
        treefmt-nix.flakeModule
        devenv.flakeModule
      ];
      systems = import inputs.systems;

      flake = {
        templates =
          let
            welcomeText = ''
              Hi! You've just created a fresh new flakes project using the
              practical-flakes-template. You can start by looking around or
              running the development environment either via direnv (`direnv allow`)

              Furthermore don't forget to rename your project using
              `rename-project . myAwesomeNewProject`

              For more info refer to
              https://github.com/tsandrini/practical-flakes-template/
            '';
          in
          {
            default = inputs.self.templates.main;
            main = {
              inherit welcomeText;
              path = ./templates/main;
              description = "Highly opinionated nix flakes starter template that focuses on modularity.";
            };

            minimal = {
              inherit welcomeText;
              path = ./templates/minimal;
              description = "Minimal version of the highly opiniated nix flakes starter template.";
            };

            isolated = {
              inherit welcomeText;
              path = ./templates/isolated;
              description = "Isolated (./nix) version of the highly opiniated nix flakes starter template.";
            };

            isolated-minimal = {
              inherit welcomeText;
              path = ./templates/isolated-minimal;
              description = "Isolated (./nix) and minimal version of the highly opiniated nix flakes starter template.";
            };

            home = {
              inherit welcomeText;
              path = ./templates/home;
              description = "Full version of the highly opiniated nix flakes starter template that includes prewired home-manager";
            };
          };
      };

      perSystem =
        { config, pkgs, ... }:
        {
          treefmt = {
            package = pkgs.treefmt;
            flakeCheck = true;
            flakeFormatter = true;
            projectRootFile = ./flake.nix;
            programs = {
              deadnix.enable = true;
              statix.enable = true;
              nixfmt-rfc-style.enable = true;
              # NOTE Choose a different formatter if you'd like to
              # nixfmt.enable = true;
              # alejandra.enable = true;

              prettier.enable = true;
              mdformat.enable = true;
              yamlfmt.enable = true;
              jsonfmt.enable = true;

              shellcheck.enable = true;
              shfmt.enable = true;
            };
          };

          devenv.shells.dev = {

            # --------------------------
            # --- ENV & SHELL & PKGS ---
            # --------------------------
            packages = with pkgs; [
              # -- NIX UTILS --
              nix-output-monitor # Processes output of Nix commands to show helpful and pretty information
              nixfmt-rfc-style # An opinionated formatter for Nix
              # NOTE Choose a different formatter if you'd like to
              # nixfmt # An opinionated formatter for Nix
              # alejandra # The Uncompromising Nix Code Formatter
              nh # Yet another nix cli helper

              # -- GIT RELATED UTILS --
              commitizen # Tool to create committing rules for projects, auto bump versions, and generate changelogs
              cz-cli # The commitizen command line utility
              fh # The official FlakeHub CLI
              gh # GitHub CLI tool

              # -- LANGUAGE RELATED UTILS --
              markdownlint-cli # Command line interface for MarkdownLint
              typos # Source code spell checker
              config.treefmt.build.wrapper # one CLI to format the code tree

              # -- (YOUR) EXTRA PKGS --
              nodePackages.prettier # Prettier is an opinionated code formatter
            ];

            enterShell = ''
              # Welcome splash text
              echo ""; echo -e "\e[1;37;42mWelcome to the practicalFlakes devshell!\e[0m"; echo ""
            '';

            # ---------------
            # --- SCRIPTS ---
            # ---------------
            scripts = {
              "rename-project".exec = ''
                find $1 \( -type d -name .git -prune \) -o -type f -print0 | xargs -0 sed -i "s/practicalFlakes/$2/g"
              '';
            };

            # -----------------
            # --- LANGUAGES ---
            # -----------------
            languages.nix.enable = true;

            # ----------------------------
            # --- PROCESSES & SERVICES ---
            # ----------------------------

            # ------------------
            # --- CONTAINERS ---
            # ------------------
            devcontainer.enable = true;

            # ----------------------
            # --- BINARY CACHING ---
            # ----------------------
            # NOTE All available hooks options are listed at
            # https://devenv.sh/reference/options/#pre-commithooks
            pre-commit = {
              hooks = {
                treefmt.enable = true;

                nil.enable = true; # Nix Language server, an incremental analysis assistant for writing in Nix.
                markdownlint.enable = true; # Markdown lint tool
                typos.enable = true; # Source code spell checker
                editorconfig-checker.enable = true; # A tool to verify that your files are in harmony with your .editorconfig

                actionlint.enable = true; # GitHub workflows linting
                commitizen.enable = true; # Commitizen is release management tool designed for teams.
              };
              settings = {
                # We pass our custom treefmt build from parts/treefmt/treefmt.nix for
                # devenv to use.
                treefmt.package = config.treefmt.build.wrapper;
              };
            };

            # --------------
            # --- FLAKES ---
            # --------------
            devenv.flakesIntegration = true;

            # This is currently needed for devenv to properly run in pure hermetic
            # mode while still being able to run processes & services and modify
            # (some parts) of the active shell.
            devenv.root =
              let
                devenvRootFileContent = builtins.readFile inputs.devenv-root.outPath;
              in
              pkgs.lib.mkIf (devenvRootFileContent != "") devenvRootFileContent;

            # ---------------------
            # --- MISCELLANEOUS ---
            # ---------------------
            difftastic.enable = true;
          };
        };
    };
}
