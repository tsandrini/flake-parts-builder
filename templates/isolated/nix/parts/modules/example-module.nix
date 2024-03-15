# --- nix/parts/modules/example-module.nix
{ localFlake }:
{
  config,
  lib,
  pkgs,
  ...
}:
with builtins;
with lib;
let
  inherit (localFlake.lib) mkOverrideAtModuleLevel;

  cfg = config.practicalFlakes.example-module;
  _ = mkOverrideAtModuleLevel;
in
{
  options.practicalFlakes.example-module = with types; {
    enable = mkEnableOption (
      mdDoc ''
        Enable the NixOS example module that enables neovim and installs git
      ''
    );
  };

  config = mkIf cfg.enable (
    mkMerge [
      # |----------------------------------------------------------------------| #
      {
        environment.systemPackages = with pkgs; [ git ];

        # By using mkOverrideAtModuleLevel we can set a sensible override
        # priority that is higher than the <nixpkgs>/modules priority, however,
        # it can still be easily changed by the end user in their host/home
        # configurations.
        programs.neovim.enable = _ true;
      }
      # |----------------------------------------------------------------------| #
    ]
  );

  meta.maintainers = with localFlake.lib.maintainers; [ ];
}
