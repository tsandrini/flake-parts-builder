# --- flake-parts/pkgs/default.nix
{ ... }:
{
  perSystem =
    { pkgs, ... }:
    {
      packages = {
        # my-custom-package = pkgs.callPackage ./my-custom-package.nix { };
      };
    };
}
