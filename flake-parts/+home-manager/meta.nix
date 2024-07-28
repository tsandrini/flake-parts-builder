# --- meta.nix
{
  description = "(Collection) Home-manager related parts.";

  inputs = { };
  dependencies = [
    "hm-modules"
    "hm-homes"
  ];
  extraTrustedPublicKeys = [ ];
  extraSubstituters = [ ];
}
