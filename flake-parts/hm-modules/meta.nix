_: {
  description = "hm-modules";

  inputs = {
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  extraTrustedPublicKeys = [ ];
  extraSubstituters = [ ];
}
