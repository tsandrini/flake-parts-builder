{
  description = "Bindings for the agenix secrets manager with prepared NixOS/HM modules ready to be used in your configurations.";

  inputs = {
    agenix = {
      url = "github:ryantm/agenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  conflicts = [ ];
  extraTrustedPublicKeys = [ ];
  extraSubstituters = [ ];
}
