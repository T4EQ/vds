{
  description = "Video Delivery System";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
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
            overlays = [ (import rust-overlay) ];
            pkgs = import nixpkgs { inherit system overlays; };
          in
          fn pkgs
        );

      vdsPackageWithPkgs =
        pkgs:
        let
          rustPlatform = pkgs.makeRustPlatform (
            let
              toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);
            in
            {
              rustc = toolchain;
              cargo = toolchain;
            }
          );
        in
        rustPlatform.buildRustPackage {
          pname = "vds";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [
            pkgs.trunk
            pkgs.wasm-bindgen-cli_0_2_106
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
    in
    rec {
      packages = forEachSystem (pkgs: rec {
        vds = vdsPackageWithPkgs pkgs;
        vds-target = vdsPackageWithPkgs pkgs.pkgsCross.aarch64-multiplatform-musl;
        default = vds;
      });

      # Much older versions of nix (from around 2022) used a different attribute to mark the default package.
      # This is included for compatibility with those versions.
      # See https://wiki.nixos.org/w/index.php?title=Flakes&oldid=7960#Output_schema
      defaultPackage = forEachSystem (pkgs: packages."${pkgs.system}".default);

      devShells = forEachSystem (pkgs: rec {
        vds = pkgs.mkShell {
          nativeBuildInputs = [
            (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
            pkgs.trunk
            pkgs.wasm-bindgen-cli_0_2_106
            pkgs.dart-sass
            pkgs.cargo-watch
            pkgs.cargo-deny
            pkgs.diesel-cli
          ];
        };

        default = vds;
      });
    };
}
