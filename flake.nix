{
  description = "Video Delivery System";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.05";
  };

  outputs =
    {
      nixpkgs,
      ...
    }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];

      forEachSystem =
        fn:
        nixpkgs.lib.genAttrs systems (
          system:
          let
            pkgs = import nixpkgs { inherit system; };
          in
          fn pkgs
        );
    in
    {
      packages = forEachSystem (pkgs: rec {
        vds = pkgs.rustPlatform.buildRustPackage {
          pname = "vds";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [
            pkgs.trunk
            pkgs.wasm-bindgen-cli
            pkgs.dart-sass
            pkgs.lld
          ];

          buildPhase = "
            runHook preBuild

            # Let stdenv handle stripping, for consistency and to not break
            # separateDebugInfo.
            export CARGO_PROFILE_RELEASE_STRIP=false

            ARGS=\"--release --offline -j $NIX_BUILD_CORES\"
            cargo run --package xtask $ARGS -- build $ARGS --target ${pkgs.stdenv.hostPlatform.rust.rustcTarget}
            runHook postBuild

            echo Finished cargo build
          ";
        };

        default = vds;
      });

      devShells = forEachSystem (pkgs: rec {
        vds = pkgs.mkShell {
          nativeBuildInputs = [
            pkgs.cargo
            pkgs.rustc
            pkgs.trunk
            pkgs.wasm-bindgen-cli
            pkgs.dart-sass
            pkgs.cargo-watch
          ];
        };

        default = vds;
      });
    };
}
