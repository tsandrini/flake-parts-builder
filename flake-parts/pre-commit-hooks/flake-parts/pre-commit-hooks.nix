# --- flake-parts/pre-commit-hooks.nix
{ inputs, lib, ... }:
{
  imports = with inputs; [ pre-commit-hooks.flakeModule ];

  perSystem =
    { config, pkgs, ... }:
    {
      pre-commit.settings =
        let
          treefmt-wrapper = if (lib.hasAttr "treefmt" config) then config.treefmt.build.wrapper else null;
        in
        {
          excludes = [ "flake.lock" ];

          hooks = {
            treefmt.enable = if (treefmt-wrapper != null) then true else false;
            treefmt.package = if (treefmt-wrapper != null) then treefmt-wrapper else pkgs.treefmt;

            nil.enable = true; # Nix Language server, an incremental analysis assistant for writing in Nix.
            markdownlint.enable = true; # Markdown lint tool

            commitizen.enable = true; # Commitizen is release management tool designed for teams.
            editorconfig-checker.enable = true; # A tool to verify that your files are in harmony with your .editorconfig
            # actionlint.enable = true; # GitHub workflows linting
            # typos.enable = true; # Source code spell checker

            # General use pre-commit hooks
            trim-trailing-whitespace.enable = true;
            mixed-line-endings.enable = true;
            end-of-file-fixer.enable = true;
            check-executables-have-shebangs.enable = true;
            check-added-large-files.enable = true;

            gitleaks = {
              enable = true;
              name = "gitleaks";
              entry = "${pkgs.gitleaks}/bin/gitleaks protect --verbose --redact --staged";
              pass_filenames = false;
            };
          };
        };
    };
}
