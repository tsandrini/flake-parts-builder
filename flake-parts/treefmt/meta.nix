_: {
  description = "Sets up the unified treefmt formatter bindings for your flake.";

  inputs = {
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  dependencies = [ "flake-root" ];
  extraTrustedPublicKeys = [ ];
  extraSubstituters = [ ];
  gitignore = [ ];
}
