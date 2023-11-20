# --- parts/shells/default.nix
{inputs, ...}: {
  imports = with inputs; [devenv.flakeModule];
  perSystem = {
    config,
    pkgs,
    inputs',
    ...
  }: {
    devenv.shells.dev = import ./dev.nix {
      inherit pkgs inputs';
      treefmt = config.treefmt.build.wrapper;
    };
  };
}
