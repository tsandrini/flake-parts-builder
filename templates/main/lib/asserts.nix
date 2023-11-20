{lib, ...}:
with lib;
with builtins; {
  /*
  Asserts that the home-manager module is installed and imported.

  *Type*: `assertHomeManagerLoaded :: AttrSet a -> (AttrSet a | Error)`

  Example:
  ```nix title="Example" linenums="1"
  config = mkIf cfg.enable (mkMerge [
    ({
      assertions = with tensorfiles.asserts;
        [ (mkIf cfg.home.enable (assertHomeManagerLoaded config)) ];
    })
   ]);
  ```
  */
  assertHomeManagerLoaded =
    # (AttrSet) An AttrSet with the already parsed NixOS config
    cfg: {
      assertion = hasAttr "home-manager" cfg;
      message = ''
        Home configuration is enabled, however, the required home-manager module is
        missing. Please, install and import home-manager for the module to work
        properly.
      '';
    };
}
