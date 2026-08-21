{
  description = "Tiny actors framework for rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Used for rust compiler
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Advisory db from rust-sec
    advisory-db = {
      url = "github:RustSec/advisory-db";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-compat,
    utils,
    naersk,
    rust-overlay,
    advisory-db,
  }:
    utils.lib.eachDefaultSystem (system: let
      crateName = "actm";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      rust = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["llvm-tools-preview"];
      };
      naersk-lib = naersk.lib."${system}".override {
        rustc = rust;
        cargo = rust;
      };
      cargo-llvm-cov = naersk-lib.buildPackage {
        pname = "cargo-llvm-cov";
        src = pkgs.fetchzip {
          url = "https://crates.io/api/v1/crates/cargo-llvm-cov/0.5.0/download";
          extension = ".tar.gz";
          sha256 = "sha256-ifnwiOuFnpryYxLgescpxN8CzgFzSZlY+RlbyW7ND6g=";
        };
      };
      cargo-nextest = naersk-lib.buildPackage {
        pname = "cargo-nextest";
        src = pkgs.fetchzip {
          url = "https://crates.io/api/v1/crates/cargo-nextest/0.9.37/download";
          extension = ".tar.gz";
          sha256 = "sha256-1tEEZipJ8GqQqESKD9664Pax4evIp+G2tOpZuh6xN3U=";
        };
      };
      devBase = with pkgs; [
        # Build tools
        openssl
        pkg-config
        rust-analyzer
        cmake
        gnuplot
        # git tooling
        gitFull
        pre-commit
        git-lfs
        git-cliff
        # Cargo addons
        cargo-llvm-cov
        cargo-nextest
        cargo-release
        cargo-udeps
        cargo-audit
        cargo-edit
        # Formatters
        nixpkgs-fmt
        python3Packages.mdformat
        # for ci reasons
        bash
        cacert
        # Sourcehut
        hut
      ];
      sharedDeps = with pkgs; [
      ];
      sharedNativeDeps = with pkgs; [
      ];
    in rec {
      # Main binary
      packages.${crateName} = naersk-lib.buildPackage {
        pname = "${crateName}";
        buildInputs = sharedDeps;
        nativeBuildInputs = sharedNativeDeps;
        root = ./.;
      };
      # binary + tests
      packages.tests.${crateName} = naersk-lib.buildPackage {
        pname = "${crateName}";
        buildInputs = sharedDeps;
        nativeBuildInputs = sharedNativeDeps;
        root = ./.;
        doCheck = true;
      };

      packages.docs.${crateName} = naersk-lib.buildPackage {
        pname = "${crateName}";
        buildInputs = sharedDeps;
        nativeBuildInputs = sharedNativeDeps;
        root = ./.;
        dontBuild = true;
        doDoc = true;
        doDocFail = true;
      };

      defaultPackage = packages.${crateName};

      # Make some things eaiser to do in CI
      packages.lints = {
        # lint formatting
        format.${crateName} = with import nixpkgs {inherit system;};
          stdenv.mkDerivation {
            name = "format lint";
            src = self;
            nativeBuildInputs = with pkgs;
              [rust-bin.stable.latest.default] ++ sharedNativeDeps;
            buildInputs = sharedDeps;
            buildPhase = "cargo fmt -- --check";
            installPhase = "mkdir -p $out; echo 'done'";
          };
        # audit against stored advisory db
        audit.${crateName} = with import nixpkgs {inherit system;};
          stdenv.mkDerivation {
            name = "format lint";
            src = self;
            nativeBuildInputs = with pkgs;
              [rust-bin.stable.latest.default cargo-audit]
              ++ sharedNativeDeps;
            buildInputs = sharedDeps;
            buildPhase = ''
              export HOME=$TMP
              mkdir -p ~/.cargo
              cp -r ${advisory-db} ~/.cargo/advisory-db
              cargo audit -n
            '';
            installPhase = "mkdir -p $out; echo 'done'";
          };
        # Clippy
        clippy.${crateName} = naersk-lib.buildPackage {
          pname = "${crateName}";
          root = ./.;
          buildInputs = sharedDeps;
          nativeBuildInputs = sharedNativeDeps;
          cargoTestCommands = old: ["cargo $cargo_options clippy"];
          doCheck = true;
          dontBuild = true;
        };
      };

      devShell = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.packages.${system};
        buildInputs = [rust] ++ devBase ++ sharedDeps ++ sharedNativeDeps;
      };

      packages.nightlyRustShell = pkgs.mkShell {
        buildInputs =
          [
            (pkgs.rust-bin.selectLatestNightlyWith (toolchain:
              toolchain.default.override {
                extensions = ["rust-src" "clippy" "llvm-tools-preview"];
              }))
          ]
          ++ devBase
          ++ sharedDeps
          ++ sharedNativeDeps;
      };
    });
}
