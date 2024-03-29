# --- nix/parts/treefmt/treefmt.nix
{ pkgs, projectPath, ... }:
{
  # treefmt is a formatting tool that saves you time: it provides
  # developers with a universal way to trigger all formatters needed for the
  # project in one place.
  # For more information refer to
  #
  # - https://numtide.github.io/treefmt/
  # - https://github.com/numtide/treefmt-nix

  package = pkgs.treefmt;
  flakeCheck = true;
  flakeFormatter = true;
  projectRootFile = projectPath + "/flake.nix";

  programs = {
    deadnix.enable = true; # Find and remove unused code in .nix source files
    statix.enable = true; # Lints and suggestions for the nix programming language
    nixfmt-rfc-style.enable = true; # An opinionated formatter for Nix
    # NOTE Choose a different formatter if you'd like to
    # nixfmt.enable = true; # An opinionated formatter for Nix
    # alejandra.enable = true; # The Uncompromising Nix Code Formatter

    prettier.enable = true; # Prettier is an opinionated code formatter
    mdformat.enable = true; # CommonMark compliant Markdown formatter
    yamlfmt.enable = true; # An extensible command line tool or library to format yaml files.
    jsonfmt.enable = true; # Formatter for JSON files
  };
}
