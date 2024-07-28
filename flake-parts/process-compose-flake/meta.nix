{
  description = "Bindings for process-compose-flake and a simple process-compose template.";

  inputs = {
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
  };

  conflicts = [ "devenv" ];
  extraTrustedPublicKeys = [ ];
  extraSubstituters = [ ];
}
