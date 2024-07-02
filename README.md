# flake-parts-builder

[![flake check](https://github.com/tsandrini/practical-flakes-template/actions/workflows/check-on-merge.yml/badge.svg)](https://github.com/tsandrini/practical-flakes-template/actions/workflows/check-on-merge.yml)
[![FlakeHub](https://github.com/tsandrini/practical-flakes-template/actions/workflows/flakehub.yml/badge.svg)](https://github.com/tsandrini/practical-flakes-template/actions/workflows/flakehub.yml)

## About

TODO links, emotes, etc...
TODO minimal dependencies
TODO zero dependency

Building a new flake project? Need a template with all the necessary
boilerplate, but none perfectly fits your needs? Just **build your own**!

## Available parts

## Using your own parts

Create a flake `myFlake` (this can be any valid flake url, for example
`nixpkgs`, `./`, `/etc/myFlake` or `github:repo/myFlake`) with a derivation
`myParts` that has parts stored at `$out/parts`, then you can simply pass

```bash
nix run github:tsandrini/flake-parts-builder -- part1,myPart1,myPart2 -I myFlake#myParts
```

## FAQ

### 1. Why not use `flake.templates` instead?

### 2. Can't we just stuff this functionality into flakeModules?
