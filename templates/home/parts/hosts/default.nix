# --- parts/hosts/default.nix
# {
#   lib,
#   inputs,
#   self,
#   projectPath,
#   withSystem,
#   ...
# }: let
#   mkHost = args: hostName: {
#     extraSpecialArgs ? {},
#     extraModules ? [],
#     extraOverlays ? [],
#     withHomeManager ? false,
#     ...
#   }: let
#     baseSpecialArgs =
#       {
#         inherit (args) system self' inputs';
#         inherit inputs self hostName projectPath;
#       }
#       // extraSpecialArgs;
#   in
#     lib.nixosSystem {
#       inherit (args) system;
#       specialArgs =
#         baseSpecialArgs
#         // {
#           inherit lib hostName;
#           host.hostName = hostName;
#         };
#       modules =
#         [
#           {
#             nixpkgs.overlays = extraOverlays;
#             nixpkgs.config.allowUnfree = true;
#             networking.hostName = hostName;
#           }
#           ./${hostName}
#         ]
#         ++ extraModules
#         ++ (
#           if withHomeManager
#           then [
#             inputs.home-manager.nixosModules.home-manager
#             {
#               home-manager = {
#                 useGlobalPkgs = true;
#                 useUserPackages = true;
#                 extraSpecialArgs = baseSpecialArgs;
#                 sharedModules = [] # TODO: add shared modules here;
#               };
#             }
#           ]
#           else []
#         );
#     };
# in {
#   flake.nixosConfigurations = {
#     exampleHost = withSystem "x86_64-linux" (args: mkHost args "exampleHost" {});
#     anotherExampleHost = withSystem "x86_64-linux" (args: mkHost args "anotherExampleHost" {
#       withHomeManager = true;
#     });
#   };
# }
{}
