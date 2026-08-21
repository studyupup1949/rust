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
      url = "github:ipetkov/crane/v0.15.0";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    srht-actions.url = "sourcehut:~onemoresuza/srht-actions";
    nix-fast-build.url = "github:Mic92/nix-fast-build";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
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
    nix-fast-build,
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
        srht-actions = srht-actions.packages.${system};
        inherit (nix-fast-build.packages.${system}) nix-fast-build;
      };

      packages = let
        p = import ./nix/packages {
          inherit pkgs;
          inherit craneLib craneLibMusl;
        };
      in
        p // {default = p.aba;};

      fenixRust =
        fenix.packages.${system}.stable;
      fenixMusl = with fenix.packages.${system};
        combine [
          stable.cargo
          stable.rustc
          targets.x86_64-unknown-linux-musl.stable.rust-std
        ];
      craneLib =
        crane.lib.${system}.overrideToolchain fenixRust.toolchain;

      craneLibMusl = crane.lib.${system}.overrideToolchain fenixMusl;
    in {
      inherit checks packages devShells;
    });
}
