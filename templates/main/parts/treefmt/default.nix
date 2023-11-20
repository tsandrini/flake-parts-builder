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
