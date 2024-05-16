{
  description = "A compiler, runtime environment and debugger for an assembly-like programming language called Alpha-Notation";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
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

            # cross compilation to windows
            toolchainWin = with inputs.fenix.packages.${system};
              combine [
                minimal.rustc
                minimal.cargo
                targets.x86_64-pc-windows-gnu.latest.rust-std
              ];
            craneLibWin = (inputs.crane.mkLib pkgs).overrideToolchain toolchainWin;
          in
          {
            devShells.default = pkgs.mkShell {
              buildInputs = with pkgs; [
                cargo
                gcc
                rustfmt
                clippy
                cargo-llvm-cov
                rustc.llvmPackages.llvm
                vhs
              ];

              # Certain Rust tools won't work without this
              # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
              # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
              RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
              LLVM_COV = "${pkgs.rustc.llvmPackages.llvm}/bin/llvm-cov";
              LLVM_PROFDATA = "${pkgs.rustc.llvmPackages.llvm}/bin/llvm-profdata";
            };

            # this shell contains required programs to be able to build the release artifacts
            # using package_release.sh
            # activate with: 'nix develop .#buildArtifact'
            devShells.buildArtifact = pkgs.mkShell {
              buildInputs = with pkgs; [
                cargo-cross
                rustup
                zip
              ];
            };

            packages = {

              default = alpha_tui;

              alpha_tui-win = craneLibWin.buildPackage {
                src = src;

                strictDepts = true;
                doCheck = false;

                CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
                RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
                  pkgs.pkgsCross.mingwW64.windows.pthreads
                ]);

                depsBuildBuild = with pkgs;
                  [
                    pkgsCross.mingwW64.stdenv.cc
                    pkgsCross.mingw32.windows.pthreads
                    # somehow the build fails if this 64 bit version is used instead of the 32 bit version
                    #pkgsCross.mingwW64.windows.pthreads
                  ];
              };
            };

          };

      };
}
