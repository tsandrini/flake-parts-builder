_: {
  description = "Process-compose-flake";

  inputs = {
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
  };

  conflicts = [ "devenv" ];
  extraTrustedPublicKeys = [ ];
  extraSubstituters = [ ];
}
