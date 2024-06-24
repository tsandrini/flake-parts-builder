{ pkgs, ... }: {
    description = "Systems flake-part";

    inputs = {
        something = "github:test/test";
        eh = pkgs.python3;
    };

    dependencies = [];
    conflicts = [];
    extraTrustedPublicKeys = [];
    extraSubstituters = [];
}
