# practicalFlakes

## Description

This flake has been generated from the
[tsandrini/practical-flakes-template](https://github.com/tsandrini/practical-flakes-template/)
project. The next steps for your development are

1. **development environment**
   - either use [direnv](https://github.com/direnv/direnv) and `direnv allow`
   - or explicitly enter the shell via
     `nix develop .#dev --override-input devenv-root "file+file://"<(printf %s "$PWD")`
1. **rename the project**
   - while not many, there are some places in the code that have a `practicalFlakes`
     identifier, you can use the `rename-project` (available in the dev environment)
     script to change these
   - `rename-project . myCoolNewProject`
