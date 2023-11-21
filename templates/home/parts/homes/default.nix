# --- parts/homes/default.nix
# {
#   lib,
#   inputs,
#   projectPath,
#   withSystem,
#   self,
#   config,
#   ...
# }: let
#   mkHome = args: home: {
#     extraSpecialArgs ? {},
#     extraModules ? [],
#   }:
#     inputs.home-manager.lib.homeManagerConfiguration {
#       inherit (args) pkgs;
#       extraSpecialArgs =
#         {
#           inherit (args) system;
#           inherit inputs home projectPath self;
#         }
#         // extraSpecialArgs;
#       modules =
#         [
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
