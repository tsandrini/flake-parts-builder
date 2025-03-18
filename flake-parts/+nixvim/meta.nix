# --- meta.nix
{
  description = "(Collection) All of the nixvim related parts.";

  inputs = { };
  dependencies = [
    "nixvim-configurations"
    "nixvim-modules"
  ];
  extraTrustedPublicKeys = [ ];
  extraSubstituters = [ ];
}
