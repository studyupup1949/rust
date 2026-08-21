# SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>
#
# SPDX-License-Identifier: ISC
{
  callPackage,
  craneLib,
  craneLibMusl,
}: let
  aba = callPackage (import ./aba) {inherit craneLib;};
  artifacts = callPackage (import ./aba/artifacts.nix) {inherit craneLib;};
  abaMusl = (aba.override {craneLib = craneLibMusl;}).overrideAttrs (_old: {
    cargoArtifacts = null;
    strictDeps = true;
    CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
    CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
  });
  dist = abaMusl.overrideAttrs (_old: {
    dontBuild = true;
    dontInstall = true;
    doCheck = false;
    doDist = true;
    distPhase = ''
      runHook preDist

      just dist-bin
      mkdir -p $out/tarballs
      cp -r *.{SHA256,tar.gz} $out/tarballs

      runHook postDist
    '';
  });
in {
  inherit aba artifacts dist abaMusl;
}
