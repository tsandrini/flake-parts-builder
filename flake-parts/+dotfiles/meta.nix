# --- meta.nix
{
  description = "(Collection) Opinionated set of parts to bootstrap your personal dotfiles!";

  inputs = { };
  dependencies = [
    "+nixos"
    "+home-manager"
    "shells"
    "pkgs"
    "overlays"
    "pre-commit-hooks"
    "treefmt"
    "systems"
  ];
  extraTrustedPublicKeys = [ ];
  extraSubstituters = [ ];
}
