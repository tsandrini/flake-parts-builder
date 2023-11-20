# --- parts/pkgs/default.nix
{lib, ...}: {
  perSystem = {
    pkgs,
    system,
    ...
  }: {
    packages = {
      example-package = pkgs.callPackage ./example-package {inherit lib system;};
    };
  };
}
