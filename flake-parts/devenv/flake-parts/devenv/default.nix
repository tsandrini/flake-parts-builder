# --- flake-parts/devenv/default.nix
{ inputs, ... }:
{
  imports = with inputs; [ devenv.flakeModule ];

  perSystem =
    {
      config,
      pkgs,
      system,
      ...
    }:
    {
      devenv.shells.dev = import ./dev.nix {
        inherit pkgs system;
        inherit (inputs) devenv-root;
        treefmt = config.treefmt.build.wrapper;
      };
    };
}
