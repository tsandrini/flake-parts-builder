# --- flake-parts/deploy-rs/default.nix
{ inputs, config, ... }:
let
  inherit (inputs) deploy-rs;

  hostPath =
    system: name: deploy-rs.lib.${system}.activate.nixos config.flake.nixosConfigurations.${name};
in
{
  flake.deploy.nodes = {
    # NOTE example node configuration
    # "myExampleNode" = {
    #   hostname = "10.0.0.1";
    #
    #   profiles.system = {
    #     sshUser = "admin";
    #     user = "root";
    #
    #     autoRollback = true;
    #     magicRollback = true;
    #
    #     # TODO specify flake-parts attribute of your node configuration
    #     path = hostPath "x86_64-linux" "myExampleNode";
    #   };
    # };
  };

  # NOTE This way we can automatically enable checks for all nodes
  flake.checks = builtins.mapAttrs (
    _system: deployLib: deployLib.deployChecks config.flake.deploy
  ) deploy-rs.lib;
}
