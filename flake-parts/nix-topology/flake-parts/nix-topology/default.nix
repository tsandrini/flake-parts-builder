# --- flake-parts/nix-topology/default.nix
{
  inputs,
  lib,
  self,
  ...
}:
let
  inherit (inputs.flake-parts.lib) importApply;
  localFlake = self;
in
{
  imports = with inputs; [ nix-topology.flakeModule ];

  perSystem =
    { ... }:
    {
      topology.modules = [
        { inherit (localFlake) nixosConfigurations; }
        (importApply ./topology.nix { inherit localFlake; })
      ];
    };
}
