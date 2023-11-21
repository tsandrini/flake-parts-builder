# --- parts/modules/default.nix
_: {
  imports = [
    ./nixos
    ./home-manager
  ];
}
