# --- nix/parts/shells/default.nix
{
  inputs,
  projectPath,
  ...
}: {
  imports = with inputs; [devenv.flakeModule];
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    devenv.shells.dev = import ./dev.nix {
      inherit pkgs projectPath;
      treefmt = config.treefmt.build.wrapper;
    };
  };
}
