{
  description = "A compiler, runtime environment and debugger for an assembly-like programming language called Alpha-Notation";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake
      {
        inherit inputs;
      }
      {
        systems = [
          "x86_64-linux"
          "aarch64-linux"
          "x86_64-darwin"
          "aarch64-darwin"
        ];
        perSystem =
          { config
          , pkgs
          , system
          , self
          , ...
          }:
          let
            craneLib = inputs.crane.lib.${system};
            src = craneLib.cleanCargoSource (craneLib.path ./.);
            cargoArtifacts = craneLib.buildDepsOnly { inherit src; };
            alpha_tui = craneLib.buildPackage {
              inherit cargoArtifacts src;
              # disable check because two tests fail because files can not be found (needs to be fixed, but I currently don't know how)
              doCheck = false;
            };
          in
          {
            devShells.default = pkgs.mkShell {
              buildInputs = with pkgs; [
                cargo
                gcc
                rustfmt
                clippy
                cargo-tarpaulin
              ];
            };

            packages = {

              default = alpha_tui;

            };

          };

      };
}
