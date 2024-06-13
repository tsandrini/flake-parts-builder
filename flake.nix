# --- flake.nix
{
  description = "flake-parts-builder - TODO";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    let
      inherit (inputs.nixpkgs) lib;
    in
    flake-parts.lib.mkFlake { inherit inputs; } {

      systems = import inputs.systems;

      flake.lib = rec {
        flatten = attrs: lib.collect (x: !lib.isAttrs x) attrs;

        mapFilterAttrs =
          pred: f: attrs:
          lib.filterAttrs pred (lib.mapAttrs' f attrs);

        mapModules =
          dir: fn:
          mapFilterAttrs (n: v: v != null && !(lib.hasPrefix "_" n) && !(lib.lib.hasPrefix ".git" n)) (
            n: v:
            let
              path = "${toString dir}/${n}";
            in
            if v == "directory" && builtins.pathExists "${path}/default.nix" then
              lib.nameValuePair n (fn path)
            else if v == "directory" then
              lib.nameValuePair n (mapModules path fn)
            else if v == "regular" && n != "default.nix" && lib.hasSuffix ".nix" n then
              lib.nameValuePair (lib.removeSuffix ".nix" n) (fn path)
            else
              lib.nameValuePair "" null
          ) (builtins.readDir dir);

        loadParts = dir: flatten (mapModules dir (x: x));
      };

      perSystem =
        {
          config,
          pkgs,
          system,
          ...
        }:
        {

          packages = rec {
            # builder =
            default = builder;

            builder = pkgs.rustPlatform.buildRustPackage {
              name = "flake-parts-builder";
              version = "1.0.0";

              src = ./.;

              nativeBuildInputs = with pkgs; [ pkg-config ];
              buildInputs = with pkgs; [ openssl ];

              cargoSha256 = "sha256-N/nUBwSVGBotGuAXw1ywxQpgssT+9So6g2eeEZ6dIZA=";
            };

            parts = pkgs.stdenv.mkDerivation {
              name = "parts";
              version = "1.0.0";
              src = ./assets/parts;

              dontBuild = true;

              installPhase = ''
                mkdir -p $out/parts
                cp -rv $src/* $out/parts
              '';

              meta = with pkgs.lib; {
                homepage = "TODO";
                description = "TODO";
                license = licenses.mit;
                platforms = [ system ];
                maintainers = [ ];
              };
            };
          };

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustc
              cargo
              clippy
              rustfmt
              rust-analyzer
            ];
          };
        };
    };
}
