# --- nix/lib/_bootstrap-lib.nix
{lib, ...}:
with lib;
with builtins; rec {
  # This file should provide the bare minimum to bootstrap the lib, namely the
  # mapModules' function to enable properly loading the library files and its
  # functions

  mapFilterAttrs' = pred: f: attrs: filterAttrs pred (mapAttrs' f attrs);

  mapModules' = dir: fn:
    mapFilterAttrs'
    (n: v: v != null && !(hasPrefix "_" n) && !(hasPrefix ".git" n)) (n: v: let
      path = "${toString dir}/${n}";
    in
      if v == "directory" && pathExists "${path}/default.nix"
      then nameValuePair n (fn path)
      else if v == "directory"
      then nameValuePair n (mapModules' path fn)
      else if v == "regular" && n != "default.nix" && hasSuffix ".nix" n
      then nameValuePair (removeSuffix ".nix" n) (fn path)
      else nameValuePair "" null) (readDir dir);
}
