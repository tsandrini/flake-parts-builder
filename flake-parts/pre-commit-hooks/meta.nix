_: {
  description = "Bindings for pre-commit-hooks.nix and a simple pre-commit-hook template.";

  inputs = {
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
  };
  conflicts = [ "devenv" ];
  extraTrustedPublicKeys = [
    "pre-commit-hooks.cachix.org-1:Pkk3Panw5AW24TOv6kz3PvLhlH8puAsJTBbOPmBo7Rc="
  ];
  extraSubstituters = [ "https://pre-commit-hooks.cachix.org/" ];
}
