# --- flake-parts/hosts/default.nix
{
  lib,
  inputs,
  withSystem,
  config,
  ...
}:
let
  mkHost =
    args: hostName:
    {
      extraSpecialArgs ? { },
      extraModules ? [ ],
      extraOverlays ? [ ],
      withHomeManager ? false,
      ...
    }:
    let
      baseSpecialArgs = {
        inherit (args) system;
        inherit inputs hostName;
      } // extraSpecialArgs;
    in
    inputs.nixpkgs.lib.nixosSystem {
      inherit (args) system;
      specialArgs = baseSpecialArgs // {
        inherit lib hostName;
        host.hostName = hostName;
      };
      modules =
        [
          {
            nixpkgs.overlays = extraOverlays;
            nixpkgs.config.allowUnfree = true;
            networking.hostName = hostName;
          }
          ./${hostName}
        ]
        ++ extraModules
        # NOTE You can also load all of your defined modules in the
        # following manner
        #
        # ++ (lib.attrValues config.flake.nixosModules)
        ++ (
          if (withHomeManager && (lib.hasAttr "home-manager" inputs)) then
            [
              inputs.home-manager.nixosModules.home-manager
              {
                home-manager = {
                  useGlobalPkgs = true;
                  useUserPackages = true;
                  extraSpecialArgs = baseSpecialArgs;

                  # NOTE You can also load all of your defined modules in the
                  # following manner
                  #
                  # sharedModules = lib.attrValues config.flake.homeModules;
                };
              }
            ]
          else
            [ ]
        );
    };
in
{
  flake.nixosConfigurations = {
    # myExampleHost = withSystem "x86_64-linux" (
    #   args:
    #   mkHost args "spinorbundle" {
    #     withHomeManager = true;
    #     extraOverlays = with inputs; [
    #       neovim-nightly-overlay.overlays.default
    #       (final: _prev: { nur = import inputs.nur { pkgs = final; }; })
    #     ];
    #   }
    # );
  };
}
