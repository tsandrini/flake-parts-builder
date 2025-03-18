# --- flake-parts/modules/nixvim/default.nix
{
  lib,
  self,
  inputs,
  ...
}:
let
  inherit (lib) mkOption types;
  inherit (inputs.flake-parts.lib) importApply;
  localFlake = self;
in
{
  options.flake.nixvimModules = mkOption {
    type = types.lazyAttrsOf types.unspecified;
    default = { };
  };

  config.flake.nixvimModules = {
    # NOTE Dogfooding your modules with `importApply` will make them more
    # reusable even outside of your flake. For more info see
    # https://flake.parts/dogfood-a-reusable-module#example-with-importapply

    # auto_cmds = importApply ./auto_cmds.nix { inherit localFlake; };
    # keymaps = importApply ./keymaps.nix { inherit localFlake; };
    # settings = importApply ./settings.nix { inherit localFlake; };
  };
}
