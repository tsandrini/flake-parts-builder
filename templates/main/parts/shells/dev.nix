# --- parts/shells/dev.nix
{
  pkgs,
  treefmt,
  devenv-root,
  ...
}:
{
  # DEVENV:  Fast, Declarative, Reproducible, and Composable Developer
  # Environments using Nix developed by Cachix. For more information refer to
  #
  # - https://devenv.sh/
  # - https://github.com/cachix/devenv

  # --------------------------
  # --- ENV & SHELL & PKGS ---
  # --------------------------
  packages = with pkgs; [
    # -- NIX UTILS --
    nix-output-monitor # Processes output of Nix commands to show helpful and pretty information
    nixfmt-rfc-style # An opinionated formatter for Nix
    # NOTE Choose a different formatter if you'd like to
    # nixfmt # An opinionated formatter for Nix
    # alejandra # The Uncompromising Nix Code Formatter
    nh # Yet another nix cli helper

    # -- GIT RELATED UTILS --
    # commitizen # Tool to create committing rules for projects, auto bump versions, and generate changelogs
    # cz-cli # The commitizen command line utility
    # fh # The official FlakeHub CLI
    # gh # GitHub CLI tool

    # -- BASE LANG UTILS --
    markdownlint-cli # Command line interface for MarkdownLint
    typos # Source code spell checker
    treefmt # one CLI to format the code tree

    # -- (YOUR) EXTRA PKGS --
  ];

  # env = {
  #   MYUSER = "user";
  #   # ...
  # };

  # NOTE If you'd like to automatically load a .env file you can use these
  # following expressions
  #
  # dotenv.enable = true;
  # dotenv.filename = ".env.development";

  enterShell = ''
    # Welcome splash text
    echo ""; echo -e "\e[1;37;42mWelcome to the practicalFlakes devshell!\e[0m"; echo ""
  '';

  # ---------------
  # --- SCRIPTS ---
  # ---------------
  scripts = {
    "rename-project".exec = ''
      find $1 \( -type d -name .git -prune \) -o -type f -print0 | xargs -0 sed -i "s/practicalFlakes/$2/g"
    '';
  };

  # -----------------
  # --- LANGUAGES ---
  # -----------------
  languages.nix.enable = true;

  # NOTE You can enable additional language support in the following manner
  # languages.python.enable = true;
  # languages.python.version = "3.11.3";

  # languages.rust.enable = true;
  # languages.rust.channel = "stable";

  # ----------------------------
  # --- PROCESSES & SERVICES ---
  # ----------------------------

  # ------------------
  # --- CONTAINERS ---
  # ------------------
  devcontainer.enable = true;

  # ----------------------
  # --- BINARY CACHING ---
  # ----------------------
  # NOTE Here you can configure automatic cachix binary cache pulling & pushing
  # to/from a cache named "mycache"
  #
  # cachix.pull = [ "pre-commit-hooks" ];
  # cachix.push = "mycache";

  # ------------------------
  # --- PRE-COMMIT HOOKS ---
  # ------------------------
  # NOTE All available hooks options are listed at
  # https://devenv.sh/reference/options/#pre-commithooks
  pre-commit = {
    hooks = {
      treefmt.enable = true;
      # We pass our custom treefmt build from parts/treefmt/treefmt.nix for
      # devenv to use.
      treefmt.package = treefmt;

      nil.enable = true; # Nix Language server, an incremental analysis assistant for writing in Nix.
      editorconfig-checker.enable = true; # A tool to verify that your files are in harmony with your .editorconfig
      markdownlint.enable = true; # Markdown lint tool
      # typos.enable = true; # Source code spell checker

      # actionlint.enable = true; # GitHub workflows linting
      # commitizen.enable = true; # Commitizen is release management tool designed for teams.
    };
  };

  # --------------
  # --- FLAKES ---
  # --------------
  devenv.flakesIntegration = true;

  # This is currently needed for devenv to properly run in pure hermetic
  # mode while still being able to run processes & services and modify
  # (some parts) of the active shell.
  devenv.root =
    let
      devenvRootFileContent = builtins.readFile devenv-root.outPath;
    in
    pkgs.lib.mkIf (devenvRootFileContent != "") devenvRootFileContent;

  # ---------------------
  # --- MISCELLANEOUS ---
  # ---------------------
  difftastic.enable = true;
}
