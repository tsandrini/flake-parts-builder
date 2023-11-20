# --- parts/hosts/default.nix
_: {
  flake.nixosConfigurations = {
    # exampleHost = withSystem "x86_64-linux" (args: mkHost args "exampleHost" {});
  };
}
