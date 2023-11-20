{
  description = "TODO Add description :sunglasses:";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # dev and formatting
    treefmt-nix.url = "github:numtide/treefmt-nix";
    devenv.url = "github:cachix/devenv";
    mk-shell-bin.url = "github:rrbutani/nix-mk-shell-bin";
    nix2container = {
      url = "github:nlewo/nix2container";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = inputs @ {flake-parts, ...}: let
    inherit (inputs) nixpkgs;
    inherit (lib.practicalFlakes) mapModules mkNixpkgs flatten;

    projectPath = ./.;

    # TODO change name if needed
    lib = nixpkgs.lib.extend (self: _super: {
      practicalFlakes = import ./lib {
        inherit inputs projectPath;
        pkgs = nixpkgs;
        lib = self;
      };
    });
    specialArgs = {inherit lib projectPath;};
  in
    flake-parts.lib.mkFlake {inherit inputs specialArgs;} {
      imports = flatten (mapModules ./parts (x: x));

      systems = import inputs.systems;
      flake.lib = lib.tensorfiles;

      perSystem = {
        system,
        pkgs,
        ...
      }: {
        _module.args.pkgs = mkNixpkgs nixpkgs system [];
      };
    };
}
