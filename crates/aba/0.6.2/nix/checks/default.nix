# SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>
#
# SPDX-License-Identifier: ISC

{
  callPackage,
  craneLib,
  advisory-db,
  lib,
}: let
  src = import ../packages/aba/source.nix {inherit lib craneLib;};
  cargoArtifacts = import ../packages/aba/artifacts.nix {inherit lib craneLib;};
  checks = {
    cargoFmt = craneLib.cargoFmt {inherit src;};
    cargoAudit = craneLib.cargoAudit {inherit src advisory-db;};
    cargoClippy = craneLib.cargoClippy {
      inherit src cargoArtifacts;
      cargoClippyExtraArgs = "-- --deny warnings";
    };
  };
in {
  inherit (checks) cargoFmt cargoAudit cargoClippy;
}
