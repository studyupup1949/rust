# SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>
#
# SPDX-License-Identifier: ISC
{
  description = "An address book for aerc";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane/v0.14.3";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    srht-actions.url = "sourcehut:~onemoresuza/srht-actions";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = {
    self,
    nixpkgs,
    advisory-db,
    crane,
    fenix,
    flake-utils,
    srht-actions,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      inherit (pkgs) lib;

      checks = import ./nix/checks {
        inherit craneLib advisory-db;
        inherit (pkgs) lib callPackage;
      };

      devShells = import ./nix/devShells {
        inherit craneLib pkgs fenixRust;
      };

      packages = let
        p = import ./nix/packages {
          inherit (pkgs) callPackage;
          inherit craneLib craneLibMusl;
        };
        pWithDefault = p // {default = p.aba;};
      in
        pWithDefault;

      fenixRust =
        fenix.packages.${system}.stable;
      craneLib =
        crane.lib.${system}.overrideToolchain fenixRust.toolchain;

      muslPkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      rustToolchain = muslPkgs.rust-bin.stable.latest.default.override {
        targets = ["x86_64-unknown-linux-musl"];
      };
      craneLibMusl = (crane.mkLib muslPkgs).overrideToolchain rustToolchain;
    in {
      inherit checks packages devShells;
      apps = {
        inherit (srht-actions.apps.${system}) pushToCachix;
        uploadArtifacts = let
          drv = srht-actions.packages.${system}.uploadArtifacts.override {
            extraRuntimeInputs = [
              pkgs.coreutils
              fenixRust.cargo
            ];
          };
        in {
          type = "app";
          program = lib.getExe drv;
        };
        genGitTagAndChangelog = let
          drv = srht-actions.packages.${system}.genGitTagAndChangelog.override {
            extraRuntimeInputs = [
              pkgs.coreutils
              fenixRust.cargo
            ];
          };
        in {
          type = "app";
          program = lib.getExe drv;
        };
      };
    });
}
