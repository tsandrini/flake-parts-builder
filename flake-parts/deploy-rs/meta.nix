{
  description = "A Simple multi-profile Nix-flake deploy tool.";

  inputs = {
    deploy-rs.url = "github:serokell/deploy-rs";
  };
  conflicts = [ ];
  extraTrustedPublicKeys = [ ];
  extraSubstituters = [ ];
}
