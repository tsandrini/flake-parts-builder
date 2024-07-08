# --- meta.nix
_: {
  description = "(Collection) NixOS related parts.";

  inputs = { };
  dependencies = [
    "nixos-hosts"
    "nixos-modules"
  ];
  extraTrustedPublicKeys = [ ];
  extraSubstituters = [ ];
}
