# --- flake-parts/process-compose-flake/default.nix
{ inputs, lib, ... }:
{
  imports = with inputs; [ process-compose-flake.flakeModule ];

  perSystem =
    { config, pkgs, ... }:
    {
      process-compose = {
        default = config.process-compose.dev-process;

        dev-process = import ./dev.nix { inherit pkgs lib; };
      };
    };
}
