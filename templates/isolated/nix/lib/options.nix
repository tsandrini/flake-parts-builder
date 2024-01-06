# --- nix/lib/options.nix
_: {
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
