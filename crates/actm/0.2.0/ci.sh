#!/usr/bin/env bash

###
## This script replicates the same steps as ci, except for cargo audit
###

# Turn on the guard rails
set -exuo pipefail

## TODO use subshell magic to make a nice interface here

# Lint the formatting
nix build .#lints.format.actm -L
# Audit it
nix develop -c cargo audit
# Run clippy
nix build .#lints.clippy.actm -L
# Build it
nix build .#actm -L
# Test it
nix develop -c cargo nextest run
nix develop -c cargo test --doc
# Document it
nix build .#docs.actm.doc -L
