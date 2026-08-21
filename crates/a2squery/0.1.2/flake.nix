{
  description = "CLI tool for querying source engine dedicated servers";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, }: flake-utils.lib.eachDefaultSystem (system:
  let
    pkgs = nixpkgs.legacyPackages.${system};
  in rec {
    packages.a2sQuery = pkgs.rustPlatform.buildRustPackage {
      pname = "a2squery";
      version = "0.1.2";

      src = ./.;

      cargoLock = {
        lockFile = ./Cargo.lock;
      };

      buildInputs = [];
    };
    defaultPackage = packages.a2sQuery;
    apps.a2sQuery = flake-utils.lib.mkApp {
      drv = packages.a2sQuery;
    };
    defaultApp = apps.a2sQuery;
  });
}
