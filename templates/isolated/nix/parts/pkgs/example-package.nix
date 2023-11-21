# --- nix/parts/pkgs/example-package.nix
{
  lib,
  system,
  stdenv,
  ...
}:
stdenv.mkDerivation rec {
  name = "example-pkg";
  version = "v0.1.0";

  src = ./.;
  installPhase = ''
    mkdir -p $out
    echo "Hello, this is an example package" > $out/example.txt
  '';

  meta = with lib; {
    description = "Example package";
    license = licenses.mit;
    platforms = [system];
    maintainers = [];
  };
}
