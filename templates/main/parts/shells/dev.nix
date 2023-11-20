# --- parts/shells/dev.nix
{
  pkgs,
  treefmt,
  ...
}: {
  packages = with pkgs; [
    # -- greeting --
    cowsay
    fortune
    lolcat
    # -- nix --
    nil # LSP
    alejandra # formatting
    statix # static code analysis
    deadnix # find dead nix code
    nix-output-monitor # readable derivation outputs
    # -- misc --
    markdownlint-cli # markdown linting
    nodePackages.prettier
    treefmt
  ];

  languages.nix.enable = true;
  difftastic.enable = true;
  devcontainer.enable = true; # if anyone needs it
  devenv.flakesIntegration = true;

  pre-commit = {
    hooks = {
      treefmt.enable = true;
      # Everything below is stuff that is missing from treefmt
      nil.enable = true;
      markdownlint.enable = true;
      actionlint.enable = true;
    };
    settings = {
      treefmt.package = treefmt;
    };
  };

  scripts = {
    "rename-project".exec = ''
      find $1 \( -type d -name .git -prune \) -o -type f -print0 | xargs -0 sed -i "s/practicalFlakes/$2/g"
    '';
  };

  enterShell = ''
    echo ""
    echo "~~ Welcome to the practicalFlakes devshell! ~~

    [Fortune of the Day] $(fortune)" | cowsay -W 120 -T "U " | lolcat -F 0.3 -p 10 -t
    echo ""
  '';
}
