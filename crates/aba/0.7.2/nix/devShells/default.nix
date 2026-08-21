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
      jq
    ];
  };
  #
  # Needed for:
  #   1. linting the repo (reuse);
  #   1. setting up cachix binary caches (cachix); and
  #   1. publishing new versions of the crate to crates.io (fenixRust.cargo)
  ci = pkgs.mkShell {
    name = "aba-ci";
    packages = let
      getVersion = pkgs.writeShellApplication {
        name = "get-version";
        runtimeInputs = [pkgs.jq fenixRust.cargo fenixRust.rustc];
        text = ''
          cargo metadata --no-deps --format-version=1 \
            | jq -r '."packages"[0]."version"'
        '';
      };
    in
      (with pkgs; [reuse cachix])
      ++ (with fenixRust; [cargo rustc])
      ++ [getVersion];
  };
in {
  inherit default ci;
}
