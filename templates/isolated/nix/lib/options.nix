# --- nix/lib/options.nix
{lib, ...}:
with lib;
with lib.types;
with builtins; rec {
  # Example function
  # -----------------
  # /*
  # Creates an enableOption (ie `mkEnableOption`), however, already
  # preenabled.
  #
  # *Type*: `String -> Option`
  # */
  # mkAlreadyEnabledOption = description:
  #   (mkEnableOption description)
  #   // {
  #     default = true;
  #   };
}
