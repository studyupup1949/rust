# SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>
#
# SPDX-License-Identifier: ISC

{
  lib,
  craneLib,
}: let
  scdocFilter = path: _type:
    builtins.match ".*\\.[[:digit:]]\\.scd$" path != null;
  justFilter = path: _type:
    lib.strings.toLower (builtins.baseNameOf path) == "justfile";
  filter = path: type:
    (scdocFilter path type)
    || (justFilter path type)
    || (craneLib.filterCargoSources path type);
in
  lib.cleanSourceWith {
    src = craneLib.path ../../../.;
    inherit filter;
  }
