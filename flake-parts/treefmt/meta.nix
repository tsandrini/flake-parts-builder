_: {
  description = "Sets up the unified treefmt formatter bindings for your flake.";

  inputs = {
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  dependencies = [ "flake-root" ];
  extraTrustedPublicKeys = [ "numtide.cachix.org-1:2ps1kLBUWjxIneOy1Ik6cQjb41X0iXVXeHigGmycPPE=" ];
  extraSubstituters = [ "https://numtide.cachix.org" ];
}
