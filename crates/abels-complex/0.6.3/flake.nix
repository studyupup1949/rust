{
  description = "abels-complex";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem = {
        pkgs,
        system,
        ...
      }: {
        devShells.default = with pkgs;
          mkShell {
            nativeBuildInputs = [
              rustc
              cargo
              rust-analyzer
              rustfmt
              clippy
            ];
          };
      };
    };
}
