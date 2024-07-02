# --- flake-parts/shells/dev.nix
{
  lib,
  mkShell,
  nil,
  statix,
  deadnix,
  nix-output-monitor,
  nixfmt-rfc-style,
  markdownlint-cli,
  writeShellScriptBin,
  treefmt-wrapper ? null,
  dev-process ? null,
  pre-commit ? null,
}:
let
  scripts = {
    rename-project = writeShellScriptBin "rename-project" ''
      find $1 \( -type d -name .git -prune \) -o -type f -print0 | xargs -0 sed -i "s/NAMEPLACEHOLDER/$2/g"
    '';
  };

  env = {
    # MY_ENV_VAR = "Hello, World!";
    # MY_OTHER_ENV_VAR = "Goodbye, World!";
  };
in
mkShell {

  packages =
    (lib.attrValues scripts)
    ++ (lib.optional (treefmt-wrapper != null) treefmt-wrapper)
    ++ (lib.optional (dev-process != null) dev-process)
    ++ [
      # -- NIX UTILS --
      nil # Yet another language server for Nix
      statix # Lints and suggestions for the nix programming language
      deadnix # Find and remove unused code in .nix source files
      nix-output-monitor # Processes output of Nix commands to show helpful and pretty information
      nixfmt-rfc-style # An opinionated formatter for Nix

      # -- GIT RELATED UTILS --
      # commitizen # Tool to create committing rules for projects, auto bump versions, and generate changelogs
      # cz-cli # The commitizen command line utility
      # fh # The official FlakeHub CLI
      # gh # GitHub CLI tool
      # gh-dash # Github Cli extension to display a dashboard with pull requests and issues

      # -- BASE LANG UTILS --
      markdownlint-cli # Command line interface for MarkdownLint
      # nodePackages.prettier # Prettier is an opinionated code formatter
      # typos # Source code spell checker

      # -- (YOUR) EXTRA PKGS --
    ];

  shellHook = ''
    ${lib.concatLines (lib.mapAttrsToList (name: value: "export ${name}=${value}") env)}
    ${lib.optionalString (pre-commit != null) pre-commit.installationScript}

    # Welcome splash text
    echo ""; echo -e "\e[1;37;42mWelcome to the NAMEPLACEHOLDER devshell!\e[0m"; echo ""
  '';
}
