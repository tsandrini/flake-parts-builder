{
  description = "PracticalFlakesTemplate - Highly opinionated nix flakes starter template that focuses on modularity.";

  inputs = {
    # Base dependencies
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # Development
    treefmt-nix.url = "github:numtide/treefmt-nix";
    devenv.url = "github:cachix/devenv";
    mk-shell-bin.url = "github:rrbutani/nix-mk-shell-bin";
    nix2container = {
      url = "github:nlewo/nix2container";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = with inputs; [treefmt-nix.flakeModule devenv.flakeModule];
      systems = import inputs.systems;

      flake = {
        templates = let
          welcomeText = ''
            Hi! You've just created a fresh new flakes project using the
            practical-flakes-template. You can start by looking around or
            running the development environment either via direnv (`direnv allow`)
            or `nix develop .#dev --impure`.

            Furthermore don't forget to rename your project using
            `rename-project . myAwesomeNewProject`

            For more info refer to
            https://github.com/tsandrini/practical-flakes-template/
          '';
        in {
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

      perSystem = {
        config,
        pkgs,
        ...
      }: {
        treefmt = {
          package = pkgs.treefmt;
          flakeCheck = true;
          flakeFormatter = true;
          projectRootFile = ./flake.nix;
          programs = {
            alejandra.enable = true;
            deadnix.enable = true;
            statix.enable = true;
            prettier.enable = true;
          };
        };

        devenv.shells.dev = {
          packages = with pkgs; [
            # -- nix --
            nil # LSP
            alejandra # formatting
            statix # static code analysis
            deadnix # find dead nix code
            nix-output-monitor # readable derivation outputs
            # -- misc --
            markdownlint-cli # markdown linting
            nodePackages.prettier
            typos # spell checking
            # -- git, flakehub --
            commitizen
            cz-cli

            config.treefmt.build.wrapper
          ];

          languages.nix.enable = true;
          difftastic.enable = true;
          devcontainer.enable = true; # if anyone needs it
          devenv.flakesIntegration = true;

          pre-commit = {
            hooks = {
              treefmt.enable = true;

              commitizen.enable = true;
              markdownlint.enable = true;
              typos.enable = true;
              actionlint.enable = true;
            };
            settings = {
              treefmt.package = config.treefmt.build.wrapper;
            };
          };
        };
      };
    };
}
