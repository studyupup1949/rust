let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };

  # Pin rather than using "latest" so we can make clippy errors sticky
  project_root = builtins.getEnv "PWD";
  toolchain = (builtins.fromTOML (builtins.readFile ("${project_root}/rust-toolchain.toml")));
  rust = pkgs.rust-bin.stable.${toolchain.toolchain.channel}.default.override {
    targets = toolchain.toolchain.targets;
    extensions = toolchain.toolchain.components;
  };
in
pkgs.mkShell {
  # Binaries to build with
  nativeBuildInputs = (with pkgs; [
    clang
    mold-wrapped
    rust
    sccache
  ]);

  shellHook = ''
    sccache --stop-server
    sccache --start-server
  '';

  RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";
  SCCACHE_CACHE_SIZE = "120G";
  RUST_BACKTRACE = 1;
  LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
}
