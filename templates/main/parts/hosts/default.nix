# --- parts/hosts/default.nix
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
#     extraOverlays ? [],
#   }:
#     lib.nixosSystem {
#       inherit (args) system;
#       specialArgs =
#         {
#           inherit (args) system self' inputs';
#           inherit inputs lib hostName projectPath;
#         }
#         // extraSpecialArgs;
#       modules =
#         [
#           {
#             nixpkgs.overlays = extraOverlays;
#             nixpkgs.config.allowUnfree = true;
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
