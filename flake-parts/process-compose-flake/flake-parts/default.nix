# --- flake-parts/process-compose-flake/default.nix
{ inputs, lib, ... }:
{
  imports = with inputs; [ process-compose-flake.flakeModule ];

  perSystem =
    { pkgs, ... }:
    {
      process-compose = rec {
        default = dev-process;

        dev-process = import ./dev.nix { inherit pkgs lib; };
      };
    };
}
