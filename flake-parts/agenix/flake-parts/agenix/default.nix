# --- flake-parts/agenix/default.nix
{
  config,
  lib,
  inputs,
  ...
}:
{
  options.secrets = with lib.types; {
    secretsPath = lib.mkOption {
      type = path;
      default = ./secrets;
      description = "Path to the actual secrets directory";
    };

    pubkeys = lib.mkOption {
      type = attrsOf (attrsOf anything);
      default = { };
      description = ''
        The resulting option that will hold the various public keys used around
        the flake.
      '';
    };

    pubkeysFile = lib.mkOption {
      type = path;
      default = ./pubkeys.nix;
      description = ''
        Path to the pubkeys file that will be used to construct the
        `secrets.pubkeys` option.
      '';
    };

    extraPubkeys = lib.mkOption {
      type = attrsOf (attrsOf anything);
      default = { };
      description = ''
        Additional public keys that will be merged into the `secrets.pubkeys`
      '';
    };
  };

  config = {
    secrets.pubkeys = (import config.agenix.pubkeysFile) // config.agenix.extraPubkeys;

    flake.nixosModules.security_agenix =
      {
        config,
        lib,
        pkgs,
        system,
        ...
      }:
      with builtins;
      with lib;
      let
        cfg = config.NAMEPLACEHOLDER.security.agenix;
      in
      {
        options.NAMEPLACEHOLDER.security.agenix = with types; {
          enable = mkEnableOption ''
            Enables NixOS module that sets up & configures the agenix secrets
            backend.

            References
            - https://github.com/ryantm/agenix
            - https://nixos.wiki/wiki/Agenix
          '';
        };

        imports = with inputs; [ agenix.nixosModules.default ];

        config = mkIf cfg.enable {
          environment.systemPackages = [
            inputs.agenix.packages.${system}.default
            pkgs.age
          ];

          age.identityPaths = [ "/etc/ssh/ssh_host_ed25519_key" ];
        };
      };

    flake.homeModules.security_agenix =
      { config, lib, ... }:
      with builtins;
      with lib;
      let
        cfg = config.NAMEPLACEHOLDER.hm.security.agenix;
      in
      {
        options.NAMEPLACEHOLDER.hm.security.agenix = with types; {
          enable = mkEnableOption ''
            Enable Home Manager module that sets up & configures the agenix
            secrets backend.

            References
            - https://github.com/ryantm/agenix
            - https://nixos.wiki/wiki/Agenix
          '';
        };

        imports = with inputs; [ agenix.homeManagerModules.default ];

        config = mkIf cfg.enable {
          age.identityPaths = [ "${config.home.homeDirectory}/.ssh/id_ed25519" ];
        };
      };
  };
}
