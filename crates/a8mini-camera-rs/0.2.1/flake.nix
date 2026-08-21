{
  description = "A8 Mini Camera Control Rust Library";

  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    pre-commit-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs = { self, nixpkgs, utils, naersk, pre-commit-hooks }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        src = pkgs.lib.cleanSource ./.;

        a8mini-camera-rs = naersk-lib.buildPackage {
          pname = "a8mini-camera-rs";
          inherit src;
        };

        pre-commit-check = pre-commit-hooks.lib.${system}.run {
          inherit src;
          hooks = {
            nixpkgs-fmt.enable = true;
            cargo-check.enable = true;
            # clippy.enable = true;
            rustfmt.enable = true;
          };
        };

      in
      {
        checks = {
          # inherit pre-commit-check;
        };

        packages = {
          inherit a8mini-camera-rs;
          default = a8mini-camera-rs;
        };

        devShells.default = with pkgs; mkShell {
          inherit (pre-commit-check) shellHook;
          inputsFrom = [ a8mini-camera-rs ];

          buildInputs = [
            curl
            cargo
            rustc
            rustfmt
            pre-commit
            rustPackages.clippy
          ] ++ pre-commit-check.enabledPackages;
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      }
    );
}

