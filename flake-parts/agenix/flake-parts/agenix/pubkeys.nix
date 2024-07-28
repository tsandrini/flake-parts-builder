# --- flake-parts/agenix/pubkeys.nix
let
  myUser = "TODO: Add your public key here";
in
{
  common = { };
  hosts = {
    myHost = {
      users = {
        root = {
          sshKey = null;
          authorizedKeys = [ ];
        };
        myUser = {
          sshKey = null;
          authorizedKeys = [ myUser ];
        };
      };
    };
  };
}
