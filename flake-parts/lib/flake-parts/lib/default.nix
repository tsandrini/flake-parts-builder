# --- flake-parts/lib/default.nix
{
  inputs,
  lib,
  self,
  ...
}:
let
  localFlake = self;
in
{
  flake.lib = {
    # modules = import ./modules { inherit localFlake lib inputs; };
    # functions = import ./functions { inherit localFlake lib; };
  };
}
