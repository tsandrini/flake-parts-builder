# --- parts/homes/default.nix
# {
#   lib,
#   inputs,
#   projectPath,
#   withSystem,
#   self,
#   ...
# }: let
#   mkHome = args: home: {
#     extraSpecialArgs ? {},
#     extraModules ? [],
#     extraOverlays ? [],
#     ...
#   }:
#     inputs.home-manager.lib.homeManagerConfiguration {
#       inherit (args) pkgs;
#       extraSpecialArgs =
#         {
#           inherit (args) system self' inputs';
#           inherit inputs home projectPath self;
#         }
#         // extraSpecialArgs;
#       modules =
#         [
#           {
#             nixpkgs.overlays = extraOverlays;
#             nixpkgs.config.allowUnfree = true;
#           }
#           ./${home}
#         ]
#         ++ extraModules;
#     };
# in {
#   options.flake.homeConfigurations = lib.mkOption {
#     type = with lib.types; lazyAttrsOf unspecified;
#     default = {};
#   };
#   config = {
#     flake.homeConfigurations = {
#       "myHost@myUser" = withSystem "x86_64-linux" (args: mkHome args "myHost@myUser" {});
#     };
#     flake.checks."x86_64-linux" = {
#       "home-myHost@myUser" = config.flake.homeConfigurations."myHost@myUser".config.home.path;
#     };
#   };
# }
{}
