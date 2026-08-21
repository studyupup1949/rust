# SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>
#
# SPDX-License-Identifier: ISC
{
  craneLib,
  pkgs,
  fenixRust,
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
  #
  # Needed for:
  #   1. linting the repo (reuse);
  #   1. setting up cachix binary caches (cachix); and
  #   1. publishing new versions of the crate to crates.io (fenixRust.cargo)
  ci = pkgs.mkShell {
    name = "aba-build";
    packages = with pkgs;
      [
        reuse
        cachix
      ]
      ++ (with fenixRust; [cargo rustc]);
  };
in {
  inherit default ci;
}
