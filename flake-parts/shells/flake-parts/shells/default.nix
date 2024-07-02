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
          treefmtCustom = if (lib.hasAttr "treefmt" config) then config.treefmt.build.wrapper else null;
        };
      };
    };
}
