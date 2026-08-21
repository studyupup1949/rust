# TRAM types

This repo contains type / schema definitions for the types used in the TRAM system.

## typeshare

This repo implements [typeshare](https://github.com/1Password/typeshare) for generating Typescript types from the Rust definitions

 1. install Rust via [rustup](https://rustup.rs/).
 2. install typeshare cli via cargo: `cargo install typeshare-cli`
 3. run the type generation command:
	- via vscode task: `generate typescript types`
	- via command: `typeshare . --lang=typescript --output-file=./tram_types.ts`

this will add create `tram_types.ts` locally, and under the TRAM frontend source (provided TRAM repo is cloned next to this one in the same folder).
