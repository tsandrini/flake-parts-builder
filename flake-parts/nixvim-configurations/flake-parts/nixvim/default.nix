# --- flake-parts/nixvim/default.nix
{
  lib,
  inputs,
  config,
  ...
}:
let
  inherit (lib) mkOption types;
  inherit (inputs.flake-parts.lib) importApply mkPerSystemOption;

  mkNixvimConfiguration =
    name: pkgs:
    {
      extraSpecialArgs ? { },
      extraModules ? [ ],
      configImportArgs ? { },
    }:
    {
      inherit pkgs extraSpecialArgs;
      module =
        { ... }:
        {
          imports =
            [
              (importApply ./${name} configImportArgs)
            ]
            # NOTE You can also load all of your defined modules in the
            # following manner
            #
            # ++ (lib.attrValues config.flake.nixvimModules)
            ++ extraModules;
        };
    };
in
{
  options.perSystem = mkPerSystemOption (_: {
    options.nixvimConfigurations = mkOption {
      type = types.lazyAttrsOf types.unspecified;
      default = { };
    };
  });

  config = {
    perSystem =
      {
        pkgs,
        config,
        system,
        ...
      }:
      let
        inherit (inputs.nixvim.lib.${system}.check) mkTestDerivationFromNixvimModule;
        inherit (inputs.nixvim.legacyPackages.${system}) makeNixvimWithModule;
      in
      {
        # NOTE Here you can define your nixvim configurations, for more 
        # specific informations and examples see
        # https://nix-community.github.io/nixvim/
        nixvimConfigurations = {
          # example-config = mkNixvimConfiguration "example-config" pkgs { };
        };

        packages = {
          nvim = config.packages.nvim-ide-config;

          # nvim-example-config = makeNixvimWithModule config.nixvimConfigurations."example-config";
        };

        checks = {
          # nvim-example-config = mkTestDerivationFromNixvimModule config.nixvimConfigurations."example-config";
        };
      };
  };
}
