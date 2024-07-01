# --- flake-parts/overlays/default.nix
{ inputs, self, ... }:
let
  localFlake = self;
in
{
  flake.overlays = {
    # myOverlay = final: prev: {
    #   myCustomSet = {};
    # };
  };
}
