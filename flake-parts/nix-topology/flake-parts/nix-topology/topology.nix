# --- flake-parts/nix-topology/topology.nix
{ localFlake }:
{ config, ... }:
let
  inherit (config.lib.topology)
    mkInternet
    mkRouter
    mkSwitch
    mkConnection
    ;
in
{
  # Add a node for the internet
  nodes.internet = mkInternet { connections = mkConnection "router" "wan1"; };

  # Add a router that we use to access the internet
  nodes.router = mkRouter "Example Router" {
    info = "Example Router";
    # image = ./images/fritzbox.png;
    interfaceGroups = [
      [
        "eth1"
        "eth2"
        "eth3"
        "eth4"
        "wifi"
      ]
      [ "wan1" ]
    ];
    connections.eth1 = mkConnection "exampleHost1" "eth0";
    connections.wifi = mkConnection "exampleHost2" "wlp3s0";
    interfaces.eth1 = {
      addresses = [ "192.168.0.1" ];
      network = "home";
    };
  };

  networks.home = {
    name = "Home";
    cidrv4 = "192.168.0.0/24";
  };
}
