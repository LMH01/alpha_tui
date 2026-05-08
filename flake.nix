{
  description = "Build a cargo project without extra checks";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, fenix, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            config = {
              allowUnsupportedSystem = true;
            };
          };

          craneLib = crane.mkLib pkgs;

          # Common arguments can be set here to avoid repeating them later
          # Note: changes here will rebuild all dependency crates
          commonArgs = {
            src = craneLib.cleanCargoSource ./.;
            strictDeps = true;

            buildInputs = [
              # Add additional build inputs here
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              pkgs.libiconv
            ];
          };

          alpha_tui = craneLib.buildPackage (commonArgs // {
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;
            # disable check because two tests fail because files can not be found (needs to be fixed, but I currently don't know how)
            doCheck = false;
            # Additional environment variables or build phases/hooks can be set
            # here *without* rebuilding all dependency crates
            # MY_CUSTOM_VAR = "some value";
          });

          alpha_tui-linux =
            let
              # Use the musl-indexed pkgs for a static build
              pkgsMusl = pkgs.pkgsCross.musl64;
              craneLibMusl = (crane.mkLib pkgs).overrideToolchain (with fenix.packages.${system};
                combine [
                  minimal.rustc
                  minimal.cargo
                  targets.x86_64-unknown-linux-musl.latest.rust-std
                ]);
            in
            craneLibMusl.buildPackage (commonArgs // {
              cargoArtifacts = craneLibMusl.buildDepsOnly commonArgs;
              doCheck = false;
              CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";

              # Force static linking for OPENSSL
              OPENSSL_STATIC = "1";

              # Ensure we use the musl versions of libraries
              buildInputs = commonArgs.buildInputs ++ [
                pkgsMusl.openssl
              ];
            });

          # cross compilation to windows
          toolchainWin = with fenix.packages.${system};
            combine [
              minimal.rustc
              minimal.cargo
              targets.x86_64-pc-windows-gnu.latest.rust-std
            ];
          craneLibWin = (crane.mkLib pkgs).overrideToolchain toolchainWin;

          alpha_tui-win = craneLibWin.buildPackage {
            src = craneLibWin.cleanCargoSource ./.;

            strictDeps = true;
            doCheck = false;

            CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";

            # fixes issues related to libring
            TARGET_CC = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}cc";

            #fixes issues related to openssl
            OPENSSL_DIR = "${pkgs.openssl.dev}";
            OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
            OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";

            depsBuildBuild = with pkgs; [
              pkgsCross.mingwW64.stdenv.cc
              pkgsCross.mingwW64.windows.pthreads
            ];
          };

        in
        {
          checks = {
            inherit alpha_tui;
          };

          devShells = {
            default = craneLib.devShell {
              # Inherit inputs from checks.
              checks = self.checks.${system};

              # Additional dev-shell environment variables can be set directly
              # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

              # Extra inputs can be added here; cargo and rustc are provided by default.
              packages = with pkgs; [
                cargo-llvm-cov
                rustc.llvmPackages.llvm
                vhs
              ];

              RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
              LLVM_COV = "${pkgs.rustc.llvmPackages.llvm}/bin/llvm-cov";
              LLVM_PROFDATA = "${pkgs.rustc.llvmPackages.llvm}/bin/llvm-profdata";
            };

            buildArtifact = craneLib.devShell {
              packages = with pkgs; [
                zip
              ];
            };
          };

          packages = {
            default = alpha_tui;

            alpha_tui-linux = alpha_tui-linux;
            alpha_tui-win = alpha_tui-win;
          };

        });
}
