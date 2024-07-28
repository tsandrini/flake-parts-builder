# --- flake-parts/process-compose-flake/dev.nix
{ pkgs, lib }:
{
  # For more info about the process-compose format seek
  # https://github.com/Platonic-Systems/process-compose-flake
  settings = {
    environment = {
      MY_ENV_VAR = "Hello from process-compose!";
    };

    processes = {
      hello.command = ''
        ${lib.getExe pkgs.hello} -g $MY_ENV_VAR
      '';
    };
  };

  preHook = ''
    echo "Running preHook"
  '';

  postHook = ''
    echo "Running postHook"
  '';
}
