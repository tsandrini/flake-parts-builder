{
  description = "Basic template for custom nix devshells (ie. `mkShell` calls) with potential bindings to other parts.";

  inputs = { };

  conflicts = [ "devenv" ];
  extraTrustedPublicKeys = [ ];
  extraSubstituters = [ ];
}
