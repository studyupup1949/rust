{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        packages.default = pkgs.hello;

        devShell = with pkgs; mkShell {
          nativeBuildInputs = [ cargo cargo-outdated cargo-zigbuild clippy gcc protobuf rust-analyzer rustc rustfmt ];

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
