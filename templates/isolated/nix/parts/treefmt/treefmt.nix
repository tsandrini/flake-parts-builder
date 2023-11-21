# --- nix/parts/treefmt/treefmt.nix
{
  pkgs,
  projectPath,
  ...
}: {
  package = pkgs.treefmt;
  flakeCheck = true;
  flakeFormatter = true;
  projectRootFile = projectPath + "/flake.nix";

  programs = {
    alejandra.enable = true;
    deadnix.enable = true;
    statix.enable = true;
    prettier.enable = true;
  };
}
