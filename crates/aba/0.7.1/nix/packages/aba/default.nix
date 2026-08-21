# SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>
#
# SPDX-License-Identifier: ISC

{
  craneLib,
  scdoc,
  just,
  lib,
}: let
  src = import ./source.nix {inherit lib craneLib;};
  cargoArtifacts = import ./artifacts.nix {inherit lib craneLib;};
in
  craneLib.buildPackage {
    inherit src cargoArtifacts;

    nativeBuildInputs = [scdoc just];

    dontUseJustBuild = true;
    dontUseJustCheck = true;
    dontUseJustInstall = true;

    postInstall = ''
      just --set PREFIX $out install-doc
    '';
  }
