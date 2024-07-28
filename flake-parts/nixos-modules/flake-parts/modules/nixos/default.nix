{ inputs, self, ... }:
let
  inherit (inputs.flake-parts.lib) importApply;
  localFlake = self;
in
{
  flake.nixosModules = {

    # NOTE Dogfooding your modules with `importApply` will make them more
    # reusable even outside of your flake. For more info see
    # https://flake.parts/dogfood-a-reusable-module#example-with-importapply

    # programs_myProgram = importApply ./programs/myProgram.nix { inherit localFlake; };
    # services_myService = importApply ./services/myService.nix { inherit localFlake inputs; };
  };
}
