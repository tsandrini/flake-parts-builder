# flake-parts-builder

[![flake check](https://github.com/tsandrini/flake-parts-builder/actions/workflows/flake-check.yml/badge.svg)](https://github.com/tsandrini/flake-parts-builder/actions/workflows/flake-check.yml)
[![FlakeHub](https://github.com/tsandrini/flake-parts-builder/actions/workflows/flakehub-publish.yml/badge.svg)](https://github.com/tsandrini/flake-parts-builder/actions/workflows/flakehub-publish.yml)
[![cachix](https://github.com/tsandrini/flake-parts-builder/actions/workflows/cachix-push.yml/badge.svg)](https://github.com/tsandrini/flake-parts-builder/actions/workflows/cachix-push.yml)
[![flake.lock update](https://github.com/tsandrini/flake-parts-builder/actions/workflows/update-flake-lock.yml/badge.svg)](https://github.com/tsandrini/flake-parts-builder/actions/workflows/update-flake-lock.yml)

## About 📝

Building a new [flake-parts](https://github.com/hercules-ci/flake-parts) project?
Need a template with all the necessary boilerplate, but none perfectly fits your
needs? Just choose the parts that you need and **build your own**!

```bash
nix run github:tsandrini/flake-parts-builder -- init -p +github,+nixos,treefmt myNewProject
```

-----

Templates defined to be used with `nix flake init -t` typically suffer from
the case of being **static** and too simple. They usually address only one
specific thing or problem domain (eg. devenv, rust, flake-parts, dotfiles, ...)
which makes the end user quickly start running into issues  when trying to
combine said domains since real life flake projects rarely require only one
such domain.

And this is what `flake-parts-builder` solves! It serves as a
**dynamic extension** to `nix flake init -t`, nothing more, nothing less!
So let's forget about `nix flake init -t` and embrace `nix run` instead :sunglasses:

-----

Okay, but what exactly does it do then?

- `flake-parts-builder init` - **initialize** a new project with all your
  required parts
- `flake-parts-builder add` - **add** new parts to an already existing
  flake-parts project
- `flake-parts-builder list` - **list** all currently available flake-parts to
  be used with the `list` and `add` subcommands

## Installation 🤖

**Disclaimer**: `flake-parts-builder` is built on top of nix
[flakes](https://wiki.nixos.org/wiki/Flakes) and
[flake-parts](https://github.com/hercules-ci/flake-parts) hence why familiarity
with flakes is a necessity. The builder also currently runs in flakes mode only
and uses flakes to parse flake-parts stores. The following "experimental"
features are then a forced requirement
`--experimental-features 'nix-command flakes'`.

*NOTE*: if enough people will be using this project I don't have any issues
with pushing it into upstream  [nixpkgs](https://github.com/NixOS/nixpkgs).

### Nix CLI

```bash
nix profile install github:tsandrini/flake-parts-builder
```

### NixOS

```nix
{
  inputs.flake-parts-builder.url = "github:tsandrini/flake-parts-builder";

  outputs = { self, nixpkgs, flake-parts-builder }: {
    # change `yourhostname` to your actual hostname
    nixosConfigurations.yourhostname = nixpkgs.lib.nixosSystem {
      # change to your system:
      system = "x86_64-linux";
      modules = [
        ./configuration.nix
        ({ system, ... }: {
           environment.systemPackages = [ flake-parts-builder.packages.${system}.default ];
        })
      ];
    };
  };
}
```

## Binary cache 💾

`flake-parts-builder` is written in Rust with minimal dependencies to ensure
safety and consistency, however, there is also a binary cache available if you'd
like to skip the build process. 

```nix
  nixConfig = {
    extra-substituters = [
      "https://tsandrini.cachix.org"
    ];
    extra-trusted-public-keys = [
      "tsandrini.cachix.org-1:t0AzIUglIqwiY+vz/WRWXrOkDZN8TwY3gk+n+UDt4gw="
    ];
  };
```

## Available parts 📂

You can list all of the available parts with the `flake-parts-builder list`
subcommand, which goes through all of the flake-parts stores passed via the `-I`
or `--include` flag. Here is the current output the list subcommand running on
only the base parts provided by this flake (note that you can disable the base
parts using `--disable-base` if you wish so)

```bash
flake-parts-builder list
```

```md
 # github:tsandrini/flake-parts-builder#flake-parts
  - +github: (Collection) GitHub related parts
  - +home-manager: (Collection) Home-manager related parts.
  - +nixos: (Collection) NixOS related parts.
  - +nixvim: (Collection) All of the nixvim related parts.
  - agenix: Bindings for the agenix secrets manager with prepared NixOS/HM modules ready to be used in your configurations.
  - deploy-rs: A Simple multi-profile Nix-flake deploy tool.
  - devenv: Flake bindings for the `github:cachix/devenv` development environment.
  - flake-root: Provides `config.flake-root` variable pointing to the root of the flake project.
  - gh-actions-cachix: Adds a simple cachix/cachix-action GitHub action workflow.
  - gh-actions-check: Adds a simple `nix flake check` GitHub action workflow.
  - gh-actions-flake-update: Adds the periodic `DeterminateSystems/update-flake-lock` GitHub action workflow.
  - gh-actions-flakehub: Adds the push to FlakeHub GitHub action workflow.
  - gh-actions-pages: Adds a GitHub action that runs `nix build .#pages` and deploys the result to GitHub pages.
  - gh-dependabot: A basic GitHub dependabot starting template.
  - gh-templates-PR: Adds a basic GitHub pull request template.
  - gh-templates-issues: Adds basic bug/feature GitHub issue templates.
  - gitlab-ci-check: Adds a simple `nix flake check` to your GitLab CI/CD pipeline.
  - hm-homes: Template for your HM homes and a handy generator for you `homeManagerConfiguration` calls.
  - hm-modules: Basic template for custom home-manager modules.
  - lib: Basic template for custom nix library functions.
  - nix-topology: Adds bindings for the `github:oddlama/nix-topology` project to generate graphs of your networks.
  - nixos-hosts: Template for your NixOS hosts and a handy generator for `lib.nixosSystem` calls.
  - nixos-modules: Basic template for custom NixOS modules.
  - nixvim-configurations: Template for Nixvim configurations to handle multiple neovim instances.
  - nixvim-modules: Basic template for reusable nixvim modules.
  - overlays: Basic template for custom nixpkgs overlays.
  - pkgs: Basic template for custom nix packages (ie derivations).
  - pre-commit-hooks: Bindings for pre-commit-hooks.nix and a simple pre-commit-hook template.
  - process-compose-flake: Bindings for process-compose-flake and a simple process-compose template.
  - shells: Basic template for custom nix devshells (ie. `mkShell` calls) with potential bindings to other parts.
  - systems: Sets up the default `systems` of flake-parts using `github:nix-systems/default`.
  - treefmt: Bindings for the treefmt formatter and a basic treefmt configuration.

 # github:tsandrini/flake-parts-builder#flake-parts-bootstrap
  - _bootstrap: (Required) Minimal set of functions used to bootstrap your flake-parts project.
```

## `flake-parts-builder.lib` API

If you'd like to remove the `./flake-parts/_bootstrap.nix` file or you'd prefer
using any of the flake-parts functionality in a different set of circumstances 
then you can use the `flake-parts-builder.lib` output that this repo exposes. You could 
then rewrite your `flake.nix` in the following manner (this, however, adds an
additional  dependency to your project)

```nix
# --- flake.nix
{
  inputs = {
    # --- BASE DEPENDENCIES ---
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts-builder.url = "github:tsandrini/flake-parts-builder";

    # --- (YOUR) EXTRA DEPENDENCIES ---
  };

  outputs =
    inputs@{ flake-parts, ... }:
    let
      inherit (inputs.nixpkgs) lib;
      inherit (inputs.flake-parts-builder.lib) loadParts;
    in
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = loadParts ./flake-parts;
    };
}
```

For more info regarding the API of any of these functions, please refer to the 
doccomments of said functions in the `flake.nix` file.

## Using your own parts 👨‍💻👩‍💻

`flake-parts-builder` was designed from the ground up with extensibility in mind.
To be able to use local parts, remote parts and cache parts in an easy manner
the CLI accepts additional flake-parts stores via the `-I` or `--include` flag
as flake derivation outputs. Meaning that you can run

```bash
flake-parts-builder init -I ./myDir#flake-parts -p shells,pkgs,my-custom-part myNewProject
```

or even remote parts stores

```bash
flake-parts-builder init -I github:org/my-flake-project#flake-parts -p shells,my-custom-remote-part myNewProject
```

Thanks to the wonders of nix, the flake-parts stores will be resolved & fetched
only once and on successive calls, they will be copied directly from you local
`/nix/store` cache.

### Custom flake-parts-stores

A **flake-part store** is any derivation that has **flake-parts** located at
`$out/flake-parts`, so for example the following snippet

```nix
stdenv.mkDerivation {
  name = "my-custom-flake-parts";
  version = "1.0.0";
  src = ./flake-parts;

  dontConfigure = true;
  dontBuild = true;
  dontCheck = true;

  installPhase = ''
    mkdir -p $out/flake-parts
    cp -rv $src/* $out/flake-parts
  '';
}
```

This flake also exposes a handy wrapper at `flake-parts-builder.lib.mkFlakeParts`,
which shortens the previous example to

```nix
mkFlakeParts {
  inherit stdenv; # NOTE: Required
  name = "my-custom-flake-parts";
  version = "1.0.0";
  src = ./flake-parts;
}
```

### Custom flake-parts 

A **flake-part** is any folder with a **meta.nix** file at its root containing
an attrset with the following structure.

```nix
{
  description = "Flake bindings for the cachix/devenv development environment.";

  inputs = {
    devenv.url = "github:cachix/devenv"
    # ....
  };
  dependencies = [ ];
  conflicts = [ "shells" ];
  extraTrustedPublicKeys = [ "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=" ];
  extraSubstituters = [ "https://devenv.cachix.org" ];
}
```

- `description`: a simple description of the provided parts printed when running
  `flake-parts-builder list`
- `inputs`: flake inputs that will be recursively merged from all required parts
  and pasted into the final `flake.nix` file
- `dependencies`: with this you can add any additional required parts for the
  initialization/addition, this can be either a full flake uri (eg.
  `github:org/my-flake-project#flake-parts/my-custom-flake-part`) or a local
  reference to some other part **included in the same flake-parts store**
  (eg. `my-custom-part`)
- `conflicts`: a specification of other potentially conflicting parts 
  (for example when handling the same functionality, like `devenv` and `shells`)
  that should abort the process in case of found conflict, note that you can
  force the initialization/addition even in case of conflict with the
  `--ignore-conflicts`
- `extraTrustedPublicKeys`: merged with all of the required parts and pasted into
  the final `flake.nix`, for security purposes they are all commented out
- `extraSubstituters`: merged with all of the required parts and pasted into the
  final `flake.nix`, for security purposes they are all commented out

## Additional questions, issues 🗣️

### How can I use a custom version of the `nix` or `nixfmt` binary?

If installed via the nix package manager, `flake-parts-builder` will use
an isolated version of `pkgs.nixVersions.stable` with
`--extra-experimental-features 'nix-command flakes'` enabled. However, if you'd
like to use a custom version instead, simply pass it via `$NIX_BIN_PATH`,
for example

```bash
NIX_BIN_PATH=/bin/patched-nix flake-parts-builder init -p +home-manager,shells myNewProject
```

The same thing works for overriding the `nixfmt` binary using the 
`NIXFMT_BIN_PATH` environment variable

```bash
NIXFMT_BIN_PATH=/bin/nixfmt-classic flake-parts-builder init -p +home-manager,shells myNewProject
```

### Why not use `flake.templates` instead?

The `flake.templates` flake output is a static property by design that needs
to point to a fixed path with fixed content known ahead of time, which makes
it heavily impractical for any kind of dynamic evaluation. One could,
given the set of parts, prepare all of the possible combinations of templates
with some patching script and
directly update the source code of `flake.nix`, however ... At the time of
this writing there are currently $27+1$ flake parts provided by this flake in
the base collection of parts, which would result in

```math
2^{28} - 1 = 268435455
```

total combinations of templates and with an average part size of
$8.59 \pm 2.60$ KB this would result in $2.14$ total terabytes of data
with just one part per template. :skull:

I hope this is enough of an answer.

### Can't we just stuff this functionality into `flakeModules`?

I totally agree there is a fine line between a reusable piece of functionality
and boilerplate template code and I personally can't think of a general enough
definition that would discern them and also be somehow useful. However, I do
believe there is a practical, clearly visible difference between them that most
programmers can just simply look and see, let's for example take ....
[devenv/dev.nix](flake-parts/devenv/flake-parts/devenv/dev.nix) or
[nix-topology/topology.nix](flake-parts/nix-topology/flake-parts/nix-topology/topology.nix)
or even
[flake-check.yml](flake-parts/gh-actions-check/.github/workflows/flake-check.yml),
you can clearly **"see"** that this isn't a good candidate for a `flakeModule`,
they are too specific, they typically represent the end user options of some
existing `flakeModule`s. Wrapping this code into another layer of modularity
doesn't make sense, since this is meant to be a piece of configuration code.

### Help! I'm experiencing an XYZ bug!

I'm sorry for the inconvenience, please run whatever is producing said bug
with these `RUST_LOG=debug RUST_BACKTRACE=full` environment variables, 
for example

``` bash
RUST_LOG=debug RUST_BACKTRACE=full flake-parts-builder add shells ./myProject
```

and paste the output into a new bug issue. Thanks! :heart:
