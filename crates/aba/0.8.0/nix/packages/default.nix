# SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>
#
# SPDX-License-Identifier: ISC
{
  pkgs,
  craneLib,
  craneLibMusl,
}: let
  inherit (pkgs) callPackage;
  aba = callPackage (import ./aba) {inherit craneLib;};
  artifacts = callPackage (import ./aba/artifacts.nix) {inherit craneLib;};
  abaMusl = (aba.override {craneLib = craneLibMusl;}).overrideAttrs (_old: {
    pname = "aba-static-musl";
    cargoArtifacts = null;
    strictDeps = true;
    env = {
      CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
      CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
    };
  });
  hutArtifacts = abaMusl.overrideAttrs (old: {
    nativeBuildInputs = old.nativeBuildInputs ++ (with pkgs; [jq]);
    installPhase = ''
      runHook preInstall

      just PREFIX=$out artifacts

      runHook postInstall
    '';
    postInstall = "";

    dontUseJustBuild = false;
    doCheck = false;
    dontFixup = true;
  });
in {
  inherit aba artifacts abaMusl hutArtifacts;
}
