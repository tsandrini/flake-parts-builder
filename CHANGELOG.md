# Changelog

## 1.0.0-b2 (2025-01-21)

### Feat

- **flake-parts**: add deploy-rs part
- **flake-parts**: update pre-commit-hooks default
- **flake-parts**: update treefmt defaults
- **bootstrap**: add shebang to .envrc
- **flake-parts**: update issue templates
- **flake-parts**: add gh-actions-dependabot
- **flake-parts**: add nix-fast-build system doc
- **gitignore**: add .pre-commit-config.yaml

### Fix

- **builder**: update old builder hash
- **flake**: fix version typo
- **issues**: correct grammar
- **flake-parts**: fix treefmt projectRootFile
- **_bootstrap**: fix mapModules function
- **builder**: use SRI cargo hash instead

### Refactor

- **.envrc**: add shebang to .envrc
- **builder**: resolve clippy warnings
- **project**: rename PR template
- **flake-parts**: rename PR template
- **flake-parts**: rename to gh-dependabot

## 1.0.0-b1 (2024-08-01)

### Feat

- **flake-parts**: add parallelism to gh-actions
- **flake**: move builder tests to a flake check
- **flake**: add cargo test to checkPhase
- **flake-parts**: add manual dispatch to gh CI
- **docs**: add cargo doc package
- **flake-parts**: add gh-actions-cachix part
- **builder**: improve perf by making meta pure
- **flake-parts**: add nix-topology
- **flake**: update cargoSha256
- **builder**: implement add command
- **builder**: add unresolved dependencies error
- **buider**: add recursive dependencies resolver
- **builder**: add proper _bootstrap to list cmd
- **builder**: nixfmt, _bootstrap, parts tuples
- **flake**: fill meta attrs, fix builder src
- **builder**: visually distiguish collections
- **flake-parts**: add README.md to bootstrap
- **flake-parts**: add flake-parts collections
- **flake-parts**: add gh-actions-flakehub
- **flake-parts**: add gh-actions-check
- **flake-parts**: add gh-actions-flake-update
- **flake-parts**: add gh-actions-pages
- **flake-parts**: add gh-template-PR
- **flake-parts**: add gh-templates-issues
- **flake-parts**: add gitlab-ci-check
- **builder**: remove itertools requirement
- **builder**: separate nix cli & flake-inputs template
- **builder**: add flake.nix inputs template to init
- **flake-parts**: add process-compose to generic .envrc
- **builder**: separate constant into a config mod
- **builder**: switch anyhow with color-eyre
- **builder**: add fs_utils & init finalization
- **builder**: add regex dep
- **builder**: decouple & modularize src
- **flake-parts**: create one modular unified .envrc
- **builder**: add diff,fs_extra,walkdir crates
- **builder**: move parts parsing into a fun
- **builder**: complete init cmd parts parsing
- **builder**: add itertools dependency
- **flake-parts**: add home-manager related parts
- **flake-parts**: add agenix part
- **flake-parts**: add git pre-commit-hooks part
- **feat-parts**: add numtide binary cache to treefmt
- **flake-parts**: add conditional process-compose to shells
- **flake-parts**: add process-compose-flake part
- **builder**: update parts.rs
- **devshell**: update devshell & add direnv integration
- **flake-parts**: add shells, process-compose and update treefmt
- **flake-parts**: add treefmt,devenv
- **parts**: add flake-root,overlays,nixos-hosts,lib
- **parts**: add _base skeleton, pkgs, nixos parts
- **parts**: add more descriptive errors & update structs
- **builder**: separate CLI logic from parts parsing
- **project**: init new builder & parts & derivations
- **project**: init v1 builder rewrite
- **flake**: update inputs & treefmt settings patch
- **treefmt**: update defaults
- **devenv**: update to v1 & add examples & remove impurities
- **flake**: update flake inputs
- **.envrc**: update nix-direnv to version 3
- **flake**: update inputs & reformat
- **envrc**: add --accept-flake-config flag to templates
- **flake**: update inputs

### Fix

- **version**: add cz & fix version number
- **gh-workflows**: fix nix-fast-build args
- **flake**: update builder cargoSha256
- **cachix-push**: fix version main -> v15
- **builder**: correct SELF_FLAKE_URI
- **builder**: fix cmd.path parsing
- **builder**: fix conflict resolution in init
- **builder**: update temporarily SELF_FLAKE_URI
- **flake**: correct cargoHash
- **builder**: fix merging strats and force flag
- **builder**: tweak template whitespacing
- **builder**: fix flake.nix template rendering
- **flake-parts**: fix process-compose-flake structure
- **flake-parts**: fix minor flake-parts issues
- **flake-parts**: fix small discovered bugs
- **flake-parts**: fix flake-root dir structure
- **flake-parts**: fix conditional treefmt in devenv part
- **envrc**: fix incorrent devshell watch filename

### Refactor

- **flake**: move docs directly to builder drv
- **builder**: fix flake.nix.tmpl formatting
- **builder**: update flake.nix tmpl comment
- **builder**: separate shared cli args
- **builder**: decouple tmpl into templates.rs
- **builder**: move missing,conflicting parts
- **flake**: localize using callPackage pattern
- **flake-parts**: remove redundant .gitignore
- **builder**: decouple init()
- **builder**: remove redundant format_module fun
- **builder**: remove redundant .gitignore
- **flake-parts**: rename _base -> _bootstrap
- **devshell**: remove WIP rust from devshell
- **main.rs**: rename cli options
- **parts.rs**: remove redundant temporary anyhow
- **flake**: remove .code-workspace vscode file
- **flake-parts**: rename parts -> flake-parts
- **parts**: rename parts -> flake-parts
- **formatting**: switch to nixfmt-rfc-style
- **modules**: use module functors to localize inputs instead of global ones

## 0.2.0 (2024-01-06)

### Feat (0.2.0)

- **flakes**: remove unnecessary lockfiles
- **.envrc**: nix_direnv_watch_file -> watch_file
- **template-flakes**: remove nixpkgs setup, add debug comment
- **templates**: add additional sensible caches & update .envrc
- **flakehub**: add project to flakehub, update examples in README
- **templates**: add home-manager template
- **templates**: add isolated and isolated-minimal templates
- **templates**: add minimal template

### Fix (0.2.0)

- **exampleFlakes**: fix typo

### Refactor (0.2.0)

- **lib**: reformat files, update examples
- **devshells**: remove unnecessary greeting bloat

## 0.1.0 (2023-11-20)

### Feat (0.1.0)

- **README**: add usage note
- **README**: add a proper README
- **main-template**: add other examples, add some documentation
- **CI**: add github and gitlab default config
- **project**: add conventional commits and changelog
- **main-template**: add main template + devenv, treefmt, examples
- **main-template**: init
- **flake**: init dev environment
