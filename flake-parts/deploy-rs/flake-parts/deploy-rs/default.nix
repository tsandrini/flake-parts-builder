# --- flake-parts/deploy-rs/default.nix
{ inputs, config, ... }:
let
  inherit (inputs) deploy-rs;
in
{
  flake.deploy.nodes = {
    "myExampleNode" = {
      hostname = "10.0.0.1";

      profiles.system = {
        sshUser = "admin";
        user = "root";

        autoRollback = true;
        magicRollback = true;

        # TODO specify flake-parts attribute of your node configuration
        path = deploy-rs.lib.x86_64-linux.activate.nixos config.flake.nixosConfigurations."myExampleNode";
      };
    };
  };

  # NOTE This way we can automatically enable checks for all nodes
  flake.checks = builtins.mapAttrs (
    _system: deployLib: deployLib.deployChecks config.flake.deploy
  ) deploy-rs.lib;
}
