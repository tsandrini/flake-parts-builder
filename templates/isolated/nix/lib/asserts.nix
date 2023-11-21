# --- nix/lib/asserts.nix
{lib, ...}:
with lib;
with builtins; {
  # Example assertion
  # -----------------
  # assertModulesLoaded =
  #   cfg: {
  #     assertion = hasAttr "practicalFlakes" cfg;
  #     message = ''
  #       Simple example assertion that checkc if the practicalFlakes namespace
  #       is present
  #     '';
  #   };
}
