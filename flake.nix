# --- flake.nix
{
  description = "Nix flakes interactive template builder based on flake-parts written in Rust.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    let
      inherit (inputs.nixpkgs) lib;

      tsandrini = {
        email = "t@tsandrini.sh";
        name = "Tomáš Sandrini";
        github = "tsandrini";
        githubId = 21975189;
      };

      /*
        Simple wrapper around `stdenv.mkDerivation` that sets up a basic 
        derivation that holds user defined flake-parts => a flake-parts store.

        NOTE: It is required to pass an instance of your `stdenv` to this 
        function.

        *Type*: `mkFlakeParts :: Attrset a -> Package a`
      */
      mkFlakeParts =
        args@{ stdenv, ... }:
        let
          finalArgs = {
            name = "flake-parts";
            version = "1.0.0-b2";

            dontConfigure = true;
            dontBuild = true;
            dontCheck = true;

            installPhase = ''
              mkdir -p $out/flake-parts
              cp -rv $src/* $out/flake-parts
            '';
          } // args;
        in
        stdenv.mkDerivation finalArgs;
    in
    flake-parts.lib.mkFlake { inherit inputs; } {

      systems = import inputs.systems;

      flake.lib = rec {
        inherit mkFlakeParts;

        /*
          Main function for recursively traversing and loading all modules 
          in a provided flake-parts  directory.

          For more information and specifics on how this function works, see the
          doccomment of the `loadModules` function below.

          *Type*: `loadParts :: Path -> { name :: String; value :: AttrSet a; }`
        */
        loadParts = dir: flatten (mapModules dir (x: x));

        /*
          Recursively flattens a nested attrset into a list of just its values.

          *Type*: `flatten :: AttrSet a -> [a]`

          Example:
          ```nix title="Example" linenums="1"
          flatten {
            keyA = 10;
            keyB = "str20";
            keyC = {
              keyD = false;
              keyE = {
                a = 10;
                b = "20";
                c = false;
              };
            };
          }
          => [ 10 "str20" false 10 "20" false ]
          ```
        */
        flatten = attrs: lib.collect (x: !lib.isAttrs x) attrs;

        /*
          Apply a map to every attribute of an attrset and then filter the resulting
          attrset based on a given predicate function.

          *Type*: `mapFilterAttrs :: (AttrSet b -> Bool) -> (AttrSet a -> AttrSet b) -> AttrSet a -> AttrSet b`
        */
        mapFilterAttrs =
          pred: f: attrs:
          lib.filterAttrs pred (lib.mapAttrs' f attrs);

        /*
          Recursively read a directory and apply a provided function to every `.nix`
          file. Returns an attrset that reflects the filenames and directory
          structure of the root.

          Notes:

          1. Files and directories starting with the `_` or `.git` prefix will be
              completely ignored.

          2. If a directory with a `myDir/default.nix` file will be encountered,
              the function will be applied to the `myDir/default.nix` file
              instead of recursively loading `myDir` and applying it to every file.

          *Type*: `mapModules :: Path -> (Path -> AttrSet a) -> { name :: String; value :: AttrSet a; }`

          Example:
          ```nix title="Example" linenums="1"
          mapModules ./modules import
            => { hardware = { moduleA = { ... }; }; system = { moduleB = { ... }; }; }

          mapModules ./hosts (host: mkHostCustomFunction myArg host)
            => { hostA = { ... }; hostB = { ... }; }
          ```
        */
        mapModules =
          dir: fn:
          mapFilterAttrs (n: v: v != null && !(lib.hasPrefix "_" n) && !(lib.hasPrefix ".git" n)) (
            n: v:
            let
              path = "${toString dir}/${n}";
            in
            if v == "directory" && builtins.pathExists "${path}/default.nix" then
              lib.nameValuePair n (fn path)
            else if v == "directory" then
              lib.nameValuePair n (mapModules path fn)
            else if v == "regular" && n != "default.nix" && lib.hasSuffix ".nix" n then
              lib.nameValuePair (lib.removeSuffix ".nix" n) (fn path)
            else
              lib.nameValuePair "" null
          ) (builtins.readDir dir);
      };

      perSystem =
        {
          config,
          pkgs,
          system,
          ...
        }:
        {
          packages = {
            default = config.packages.builder;

            builder =
              let
                package =
                  {
                    lib,
                    rustPlatform,
                    nixfmt-rfc-style,
                    nix,
                    makeWrapper,
                    tsandrini,
                  }:
                  rustPlatform.buildRustPackage {
                    name = "flake-parts-builder";
                    version = "1.0.0-b2";

                    src = [
                      ./src
                      ./Cargo.toml
                      ./Cargo.lock
                    ];

                    unpackPhase = ''
                      runHook preUnpack
                      for srcItem in $src; do
                        if [ -d "$srcItem" ]; then
                          cp -r "$srcItem" $(stripHash "$srcItem")
                        else
                          cp "$srcItem" $(stripHash "$srcItem")
                        fi
                      done
                      runHook postUnpack
                    '';

                    preCheck = ''
                      dirs=(store var var/nix var/log/nix etc home)

                      for dir in $dirs; do
                        mkdir -p "$TMPDIR/$dir"
                      done

                      export NIX_STORE_DIR=$TMPDIR/store
                      export NIX_LOCALSTATE_DIR=$TMPDIR/var
                      export NIX_STATE_DIR=$TMPDIR/var/nix
                      export NIX_LOG_DIR=$TMPDIR/var/log/nix
                      export NIX_CONF_DIR=$TMPDIR/etc
                      export HOME=$TMPDIR/home
                    '';

                    cargoHash = "sha256-nrsbjycq1FYuDnbmbC+XIMgUYvtyblUWmjANkTn5l9w=";

                    postBuild = ''
                      cargo doc --no-deps --release
                    '';

                    NIX_BIN_PATH = lib.getExe nix;
                    NIXFMT_BIN_PATH = lib.getExe nixfmt-rfc-style;

                    postInstall = ''
                      mkdir -p $out/doc
                      cp -r target/doc $out/
                    '';

                    nativeBuildInputs = [
                      makeWrapper
                    ];

                    buildInputs = [
                      nixfmt-rfc-style
                      nix
                    ];

                    # Just add required binaries to PATH, letting the Rust
                    # program's which::which handle discovery
                    postFixup = ''
                      wrapProgram $out/bin/flake-parts-builder \
                        --prefix PATH : ${lib.makeBinPath [ nix nixfmt-rfc-style ]}
                    '';

                    meta = with lib; {
                      homepage = "https://github.com/tsandrini/flake-parts-builder";
                      description = "Nix flakes interactive template builder based on flake-parts written in Rust.";
                      license = licenses.mit;
                      platforms = [ system ];
                      maintainers = [ tsandrini ];
                      mainProgram = "flake-parts-builder";
                    };
                  };
              in
              pkgs.callPackage package {
                inherit tsandrini;
                nix = pkgs.nixVersions.stable;
              };

            flake-parts =
              let
                package =
                  {
                    lib,
                    stdenv,
                    tsandrini,
                    mkFlakeParts,
                  }:
                  mkFlakeParts {
                    inherit stdenv;
                    name = "flake-parts";
                    version = "1.0.0-b2";
                    src = ./flake-parts;

                    meta = with lib; {
                      homepage = "https://github.com/tsandrini/flake-parts-builder";
                      description = "The base collection of flake-parts for the flake-parts-builder.";
                      license = licenses.mit;
                      platforms = [ system ];
                      maintainers = [ tsandrini ];
                    };
                  };
              in
              pkgs.callPackage package { inherit tsandrini mkFlakeParts; };

            flake-parts-bootstrap =
              let
                package =
                  {
                    lib,
                    stdenv,
                    tsandrini,
                    mkFlakeParts,
                  }:
                  mkFlakeParts {
                    inherit stdenv;
                    name = "flake-parts-bootstrap";
                    version = "1.0.0-b2";
                    src = ./flake-parts-bootstrap;

                    meta = with lib; {
                      homepage = "https://github.com/tsandrini/flake-parts-builder";
                      description = "The base collection of flake-parts for the flake-parts-builder.";
                      license = licenses.mit;
                      platforms = [ system ];
                      maintainers = [ tsandrini ];
                    };
                  };
              in
              pkgs.callPackage package { inherit tsandrini mkFlakeParts; };
          };

          devShells = {
            default = config.devShells.dev;

            dev =
              let
                package =
                  {
                    mkShell,
                    nil,
                    statix,
                    deadnix,
                    nix-output-monitor,
                    nixfmt-rfc-style,
                    commitizen,
                    cz-cli,
                    gh,
                    gh-dash,
                    markdownlint-cli,
                    rustc,
                    pkg-config,
                    cargo,
                    openssl,
                    cargo-audit
                  }:
                  mkShell {
                    buildInputs = [
                      # -- NIX UTILS --
                      nil # Yet another language server for Nix
                      statix # Lints and suggestions for the nix programming language
                      deadnix # Find and remove unused code in .nix source files
                      nix-output-monitor # Processes output of Nix commands to show helpful and pretty information
                      nixfmt-rfc-style # An opinionated formatter for Nix

                      # -- GIT RELATED UTILS --
                      commitizen # Tool to create committing rules for projects, auto bump versions, and generate changelogs
                      cz-cli # The commitizen command line utility
                      # fh # The official FlakeHub CLI
                      gh # GitHub CLI tool
                      gh-dash # Github Cli extension to display a dashboard with pull requests and issues

                      # -- BASE LANG UTILS --
                      markdownlint-cli # Command line interface for MarkdownLint
                      # nodePackages.prettier # Prettier is an opinionated code formatter
                      # typos # Source code spell checker

                      # -- (YOUR) EXTRA PKGS --
                      rustc
                      cargo
                      pkg-config
                      openssl
                      cargo-audit
                    ];

                    shellHook = ''
                      # Welcome splash text
                      echo ""; echo -e "\e[1;37;42mWelcome to the flake-parts-builder devshell!\e[0m"; echo ""
                    '';
                  };
              in
              pkgs.callPackage package { };
          };
        };
    };
}
