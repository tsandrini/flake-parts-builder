_: {
  description = "agenix";

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
