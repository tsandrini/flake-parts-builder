# --- nix/parts/modules/default.nix
_: {
  flake.nixosModules = {
    example-module = import ./example-module.nix;
  };
}
