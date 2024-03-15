# --- parts/modules/home-manager/default.nix
{
  inputs,
  self,
  lib,
  ...
}:
let
  inherit (inputs.flake-parts.lib) importApply;
  localFlake = self;
in
{
  options.flake.homeModules = lib.mkOption {
    type = with lib.types; lazyAttrsOf unspecified;
    default = { };
  };

  config.flake.homeModules = {
    example-module = importApply ./example-module.nix { inherit localFlake; };
  };
}
