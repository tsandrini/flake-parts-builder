_: {
  description = "Pre-commit hooks";

  inputs = {
    inputs.pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
  };
  conflicts = [ "devenv" ];
  extraTrustedPublicKeys = [
    "pre-commit-hooks.cachix.org-1:Pkk3Panw5AW24TOv6kz3PvLhlH8puAsJTBbOPmBo7Rc="
  ];
  extraSubstituters = [ "https://pre-commit-hooks.cachix.org/" ];
}
