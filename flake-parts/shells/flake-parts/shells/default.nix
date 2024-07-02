# --- flake-parts/shells/default.nix
{ lib, ... }:
{
  perSystem =
    { pkgs, config, ... }:
    {
      devshells = rec {
        default = dev;

        dev = pkgs.callPackage ./dev.nix {
          inherit lib;
          treefmt-wrapper = if (lib.hasAttr "treefmt" config) then config.treefmt.build.wrapper else null;
          dev-process = if (lib.hasAttr "process-compose" config) then config.packages.dev-process else null;
        };
      };
    };
}
