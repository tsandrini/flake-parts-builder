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
    deadnix.enable = true;
    statix.enable = true;
    prettier.enable = true;
    nixfmt-rfc-style.enable = true;
    # NOTE Choose a different formatter if you'd like to
    # nixfmt.enable = true;
    # alejandra.enable = true;
  };
}
