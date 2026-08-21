# SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>
#
# SPDX-License-Identifier: ISC

{
  craneLib,
  lib,
}: let
  src = import ./source.nix {inherit craneLib lib;};
in
  craneLib.buildDepsOnly {
    inherit src;
  }
