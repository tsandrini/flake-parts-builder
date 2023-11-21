# --- nix/parts/treefmt/default.nix
{
  projectPath,
  inputs,
  ...
}: {
  imports = with inputs; [treefmt-nix.flakeModule];

  perSystem = {pkgs, ...}: {
    treefmt = import ./treefmt.nix {inherit pkgs projectPath;};
  };
}
