# SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>
#
# SPDX-License-Identifier: ISC

{
  craneLib,
  pkgs,
}: let
  default = craneLib.devShell {
    name = "aba-default";
    packages = with pkgs; [
      scdoc
      just
      reuse
      git-cliff
      cargo-edit
    ];
  };
  build = pkgs.mkShell {
    name = "aba-build";
    packages = with pkgs; [
      reuse
      cachix
    ];
  };
in {
  inherit default build;
}
