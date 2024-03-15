# --- nix/parts/treefmt/treefmt.nix
{ pkgs, projectPath, ... }:
{
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
