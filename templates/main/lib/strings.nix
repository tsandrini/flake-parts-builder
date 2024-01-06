# --- lib/strings.nix
_: {
  # Example function
  # -----------------
  # /*
  # Given an absolute path to a file, return the dirname of that file.

  # *Type*: `dirnameFromPath :: Path -> Path`

  # Example:
  # ```nix title="Example" linenums="1"
  # dirnameFromPath "/etc/myDir/file.nix"
  #  => "/etc/myDir"
  #  ```
  # */
  # dirnameFromPath =
  #   # (Path) Absolute path to a given file
  #   dir:
  #     trivial.pipe dir [toString (strings.splitString "/") lists.last];
}
