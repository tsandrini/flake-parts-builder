# --- flake-parts/modules/home-manager/default.nix
{
  lib,
  inputs,
  self,
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
    # programs_myProgram = importApply ./programs/myProgram { inherit localFlake; };
    # services_myService = importApply ./services/myService { inherit localFlake inputs; };
  };
}
