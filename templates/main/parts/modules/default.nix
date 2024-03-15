# --- parts/modules/default.nix
{ inputs, self, ... }:
let
  inherit (inputs.flake-parts.lib) importApply;
  localFlake = self;
in
{
  flake.nixosModules = {
    example-module = importApply ./example-module.nix { inherit localFlake; };
  };
}
