# --- parts/modules/home-manager/default.nix
{lib, ...}: {
  options.flake.homeModules = lib.mkOption {
    type = with lib.types; lazyAttrsOf unspecified;
    default = {};
  };

  config.flake.homeModules = {
    example-module = import ./example-module.nix;
  };
}
