# --- meta.nix
{
  description = "(Collection) GitHub related parts";

  inputs = { };
  dependencies = [
    "gh-actions-check"
    "gh-actions-flake-update"
    "gh-templates-issues"
    "gh-templates-PR"
  ];
  extraTrustedPublicKeys = [ ];
  extraSubstituters = [ ];
}
