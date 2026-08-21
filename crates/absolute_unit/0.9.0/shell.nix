let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };

  # Pin rather than using "latest" so we can make clippy errors sticky
  # Note: stable toolchain
  rustVersion = "1.84.1";
  rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
    extensions = [
      "rust-std"
      "rustfmt"
      "rust-src" # for rust-analyzer
      "rust-analyzer"
    ];
  };

  # If we need to do macro-backtrace or other nightly only analysis
  #rust = pkgs.rust-bin.nightly.latest.default;
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
