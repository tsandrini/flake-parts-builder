# --- flake-parts/homes/default.nix
{
  lib,
  inputs,
  withSystem,
  config,
  ...
}:
let
  mkHome =
    args: home:
    {
      extraSpecialArgs ? { },
      extraModules ? [ ],
      extraOverlays ? [ ],
      ...
    }:
    inputs.home-manager.lib.homeManagerConfiguration {
      inherit (args) pkgs;
      extraSpecialArgs = {
        inherit (args) system;
        inherit inputs home;
      } // extraSpecialArgs;
      modules = [
        {
          nixpkgs.overlays = extraOverlays;
          nixpkgs.config.allowUnfree = true;
        }
        ./${home}
      ] ++ extraModules;
      # NOTE You can also load all of your defined modules in the
      # following manner
      #
      # ++ (lib.attrValues config.flake.homeModules);
    };
in
{
  options.flake.homeConfigurations = lib.mkOption {
    type = with lib.types; lazyAttrsOf unspecified;
    default = { };
  };

  config = {
    flake.homeConfigurations = {
      # "myUser@myHost" = withSystem "x86_64-linux" (
      #   args:
      #   mkHome args "myUser@myHost" {
      #     extraOverlays = with inputs; [
      #       neovim-nightly-overlay.overlays.default
      #       (final: _prev: { nur = import inputs.nur { pkgs = final; }; })
      #     ];
      # }
      # );
    };

    flake.checks."x86_64-linux" = {
      # "home-myUser@myHost" = config.flake.homeConfigurations."myUser@myHost".config.home.path;
    };
  };
}
