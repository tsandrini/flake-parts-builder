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
    typos # spell checking
    # -- git, flakehub --
    commitizen
    cz-cli

    treefmt
  ];

  languages.nix.enable = true;
  difftastic.enable = true;
  devcontainer.enable = true; # if anyone needs it
  devenv.flakesIntegration = true;

  pre-commit = {
    hooks = {
      treefmt.enable = true;
      # Everything below is stuff that I couldn't make work with treefmt
      nil.enable = true;
      commitizen.enable = true;
      markdownlint.enable = true;
      typos.enable = true;
      actionlint.enable = true;
    };
    settings = {
      treefmt.package = treefmt;
    };
  };

  enterShell = ''
    echo ""
    echo "~~ Welcome to the practical flakes devshell! ~~

    [Fortune of the Day] $(fortune)" | cowsay -W 120 -T "U " | lolcat -F 0.3 -p 10 -t
    echo ""
  '';
}
