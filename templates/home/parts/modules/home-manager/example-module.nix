# --- parts/modules/home-manager/example-module.nix
{
  config,
  lib,
  pkgs,
  self,
  ...
}:
with builtins;
with lib; let
  practicalFlakes = self.lib;
  inherit (practicalFlakes.modules) mkOverrideAtModuleLevel;

  cfg = config.practicalFlakes.hm.example-module;
  _ = mkOverrideAtModuleLevel;
in {
  options.practicalFlakes.hm.example-module = with types;
  with practicalFlakes.types; {
    enable = mkEnableOption (mdDoc ''
      Enable the NixOS example module that enables neovim and installs git
    '');
  };

  config = mkIf cfg.enable (mkMerge [
    # |----------------------------------------------------------------------| #
    {
      home.packages = with pkgs; [git];

      # By using mkOverrideAtModuleLevel we can set a sensible override
      # priority that is higher than the <nixpkgs>/modules priority, however,
      # it can still be easily changed by the end user in their host/home
      # configurations.
      programs.neovim.enable = _ true;
      home.sessionVariables.MY_AWESOME_VAR = _ "Hello, everything working!";
    }
    # |----------------------------------------------------------------------| #
  ]);

  meta.maintainers = with practicalFlakes.maintainers; [];
}
