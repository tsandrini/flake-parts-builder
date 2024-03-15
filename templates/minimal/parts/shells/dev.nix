# --- parts/shells/dev.nix
{ pkgs, treefmt, ... }:
{
  # Some sensible defaults for a seamless initial experience have been set,
  # however, feel free to modify anything you'd like.

  packages = with pkgs; [
    # -- NIX UTILS --
    nil # Yet another language server for Nix
    statix # Lints and suggestions for the nix programming language
    deadnix # Find and remove unused code in .nix source files
    nix-output-monitor # Processes output of Nix commands to show helpful and pretty information
    nixfmt-rfc-style # An opinionated formatter for Nix
    # NOTE Choose a different formatter if you'd like to
    # nixfmt # An opinionated formatter for Nix
    # alejandra # The Uncompromising Nix Code Formatter

    # -- GIT RELATED UTILS --
    # commitizen # Tool to create committing rules for projects, auto bump versions, and generate changelogs
    # cz-cli # The commitizen command line utility
    # fh # The official FlakeHub CLI
    # gh # GitHub CLI tool

    # -- LANGUAGE RELATED UTILS --
    markdownlint-cli # Command line interface for MarkdownLint
    nodePackages.prettier # Prettier is an opinionated code formatter
    typos # Source code spell checker
    treefmt # one CLI to format the code tree

    # -- NIXOS UTILS --
    nh # Yet another nix cli helper
  ];

  languages.nix.enable = true;
  difftastic.enable = true;
  devcontainer.enable = true;
  devenv.flakesIntegration = true;

  pre-commit = {
    hooks = {
      treefmt.enable = true;

      # Everything below is stuff that is missing from treefmt
      nil.enable = true;
      markdownlint.enable = true;
      typos.enable = true;
      # actionlint.enable = true; # GitHub workflows linting
      # commitizen.enable = true; # Enable if using commitizen
    };
    settings = {
      treefmt.package = treefmt;
    };
  };

  scripts = {
    "rename-project".exec = ''
      find $1 \( -type d -name .git -prune \) -o -type f -print0 | xargs -0 sed -i "s/practicalFlakes/$2/g"
    '';
  };

  enterShell = ''
    # Greeting upon devshell activation
    echo ""; echo -e "\e[1;37;42mWelcome to the practicalFlakes devshell!\e[0m"; echo ""
  '';
}
