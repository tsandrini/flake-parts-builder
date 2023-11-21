# --- nix/parts/hosts/default.nix
# {
#   lib,
#   inputs,
#   projectPath,
#   withSystem,
#   ...
# }: let
#   mkHost = args: hostName: {
#     extraSpecialArgs ? {},
#     extraModules ? [],
#   }:
#     lib.nixosSystem {
#       inherit (args) system pkgs;
#       specialArgs =
#         {
#           inherit (args) system;
#           inherit inputs lib hostName projectPath;
#         }
#         // extraSpecialArgs;
#       modules =
#         [
#           {
#             nixpkgs.pkgs = lib.mkDefault args.pkgs;
#             networking.hostName = hostName;
#           }
#           ./${hostName}
#         ]
#         ++ extraModules;
#     };
# in {
#   flake.nixosConfigurations = {
#     # exampleHost = withSystem "x86_64-linux" (args: mkHost args "exampleHost" {});
#   };
# }
{}
