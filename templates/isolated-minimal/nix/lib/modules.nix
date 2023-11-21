# --- nix/lib/modules.nix
{
  lib,
  self,
  inputs,
  ...
}: let
  inherit (self.attrsets) mapFilterAttrs;
in
  with lib;
  with builtins; rec {
    /*
    Recursively read a directory and apply a provided function to every `.nix`
    file. Returns an attrset that reflects the filenames and directory
    structure of the root.

    Notes:

     1. Files and directories starting with the `_` or `.git` prefix will be completely
        ignored.

     2. If a directory with a `myDir/default.nix` file will be encountered,
        the function will be applied to the `myDir/default.nix` file
        instead of recursively loading `myDir` and applying it to every file.

    *Type*: `mapModules :: Path -> (Path -> AttrSet a) -> { name :: String; value :: AttrSet a; }`

    Example:
    ```nix title="Example" linenums="1"
    mapModules ./modules import
      => { hardware = { moduleA = { ... }; }; system = { moduleB = { ... }; }; }

    mapModules ./hosts (host: mkHostCustomFunction myArg host)
      => { hostA = { ... }; hostB = { ... }; }
    ```
    */
    mapModules =
      # (Path) Root directory on which should the recursive mapping be applied
      dir:
      # (Path -> AttrSet a) Function that transforms node paths to their custom attrsets
      fn:
        mapFilterAttrs
        (n: v: v != null && !(hasPrefix "_" n) && !(hasPrefix ".git" n)) (n: v: let
          path = "${toString dir}/${n}";
        in
          if v == "directory" && pathExists "${path}/default.nix"
          then nameValuePair n (fn path)
          else if v == "directory"
          then nameValuePair n (mapModules path fn)
          else if v == "regular" && n != "default.nix" && hasSuffix ".nix" n
          then nameValuePair (removeSuffix ".nix" n) (fn path)
          else nameValuePair "" null) (readDir dir);

    /*
    Custom nixpkgs constructor. Its purpose is to import provided nixpkgs
    while setting the target platform and all over the needed overlays.

    *Type*: `mkNixpkgs :: AttrSet -> String -> [(AttrSet -> AttrSet -> AttrSet)] -> Attrset`

    Example:
    ```nix title="Example" linenums="1"
    mkNixpkgs inputs.nixpkgs "x86_64-linux" []
      => { ... }

    mkNixpkgs inputs.nixpkgs "aarch64-linux" [ (final: prev: {
      customPkgs = inputs.customPkgs { pkgs = final; };
    }) ]
      => { ... }
    ```
    */
    mkNixpkgs =
      # (AttrSet) Nixpkgs attrset
      pkgs:
      # (String) System string identifier (eg: "x86_64-linux", "aarch64-linux", "aarch64-darwin")
      system:
      # ([AttrSet -> AttrSet -> AttrSet]) Extra overlays that should be applied to the created pkgs
      extraOverlays:
        import pkgs {
          inherit system;
          config.allowUnfree = true;
          hostPlatform = system;
          overlays = let
            pkgsOverlay = _final: _prev: {
              practicalFlakes = inputs.self.packages.${system};
            };
          in
            [pkgsOverlay]
            ++ (attrValues inputs.self.overlays)
            ++ extraOverlays;
        };
  }
