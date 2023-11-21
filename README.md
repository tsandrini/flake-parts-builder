# practical-flakes-template

## Description

_PracticalFlakes_ are a set of highly opinionated templates to quickly bootstrap
your next [nix](https://github.com/NixOS/nix) project in
[flakes](https://nixos.wiki/wiki/Flakes) 😎

To quickly initialize a new project run

```bash
nix flake init -t github:tsandrini/practical-flakes-template
```

And you're good to go! 👍

## Features

1. Drop-in modularity using
   [flake-parts](https://github.com/hercules-ci/flake-parts) ⚙️
   - the main idea is that if you have a bunch of projects using this template you
     can simply just copy the individual `parts/` directories to share functionality
   - this way, no matter if you're developing language features, packages,
     [NixOS modules](https://nixos.wiki/wiki/NixOS_Modules),
     [home-manager](https://github.com/nix-community/home-manager) userspace,
     you can always use the same underlying structure
2. [devenv.sh](https://github.com/cachix/devenv) is awesome! 🔥
   - includes a devenv shell already preconfigured to format and lint nix
3. [treefmt](https://github.com/numtide/treefmt) is the one and only formatter
   to rule them all 🙏
4. Already preconfigured [github actions](https://docs.github.com/en/actions)
   and [gitlab CI](https://docs.gitlab.com/ee/ci/) 💪
5. Prepared for custom `lib` overrides 🤓
   - depending on what you're currently aiming to write, you might need some
     custom helpers or library functions, this template
     already set ups all the necessary boilerplate to get it all going
6. And finally, examples included 🖌️

## Usage

After a proper installation process you can enter the development environment

1. either using [direnv](https://github.com/direnv/direnv) `direnv allow`
2. or directly `nix develop .#dev --impure`

While not many, the code has some required references to the `practicalFlakes`
identifier. This can be renamed in the whole project using the script
`rename-project` (which is available in the dev environment)

```bash
rename-project . myAwesomeApp
```

You're also encouraged to update your flakes with

```bash
nix flake update
```

## Variants

There are also a few other different variants of the base template that may
be better suited for your needs

- **main**: The main, default template.
- **home**: Conceptually and structurally the same as the default template, but
  also includes prepared and preconfigured
  [home-manager](https://github.com/nix-community/home-manager) as well as
  examples of how to use it
- **minimal**: Structurally the same as the default template, but stripped of all
  of the included examples and additional prepared files
- **isolated**: Centralizes all of the nix related stuff into a `nix/` folder.
  This can be useful when you'd like to not pollute your root with stuff not
  directly tied to the code.
- **isolated-minimal**: Isolated combined with minimal, that is, structurally the
  same as minimal, however, stripped out of all the examples and unnecessary code

You can install your desired template variant using

```bash
nix flake init -t github:tsandrini/practical-flakes-template#myVariant
```

For example,
`nix flake init -t github:tsandrini/practical-flakes-template#isolated-minimal`.

## Notes

- `pkgs` are by default enabled to allow **unfree** licenses, if you'd prefer not
  to have this enabled, simply remove the line in the `lib/modules.nix:mkNixpkgs`
  helper function
