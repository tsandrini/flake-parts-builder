_: {
  description = "Sets up the default `systems` of flake-parts";

  inputs = {
    systems.url = "github:nix-systems/default";
  };

  extraTrustedPublicKeys = [ ];
  extraSubstituters = [ ];
  gitignore = [ ];
}
