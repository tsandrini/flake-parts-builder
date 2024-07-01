# --- flake-parts/_bootstrap.nix
{ lib }:
rec {
  # TODO add docs
  flatten = attrs: lib.collect (x: !lib.isAttrs x) attrs;

  # TODO add docs
  mapFilterAttrs =
    pred: f: attrs:
    lib.filterAttrs pred (lib.mapAttrs' f attrs);

  # TODO add docs
  mapModules =
    dir: fn:
    mapFilterAttrs (n: v: v != null && !(lib.hasPrefix "_" n) && !(lib.lib.hasPrefix ".git" n)) (
      n: v:
      let
        path = "${toString dir}/${n}";
      in
      if v == "directory" && builtins.pathExists "${path}/default.nix" then
        lib.nameValuePair n (fn path)
      else if v == "directory" then
        lib.nameValuePair n (mapModules path fn)
      else if v == "regular" && n != "default.nix" && lib.hasSuffix ".nix" n then
        lib.nameValuePair (lib.removeSuffix ".nix" n) (fn path)
      else
        lib.nameValuePair "" null
    ) (builtins.readDir dir);

  # TODO add docs
  loadParts = dir: flatten (mapModules dir (x: x));
}
