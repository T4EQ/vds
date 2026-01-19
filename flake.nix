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
      self,
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

      forEachSystem = fn: nixpkgs.lib.genAttrs systems (system: fn system);

      # Note that these pacakges do not obey the rust-toolchain.toml. Unfortunately, building the
      # toolchains from the rust-toolchain.toml files would take way too long. Maybe we could
      # do this if we setup a nix cache, but for now this will do.
      vdsPackageWithPkgs =
        pkgs: targetPkgs:
        let
          # Get git revision. Nix removes the .git repository when building so that the build reproducibility
          # does not get affected. Luckily, it also provides a way for us to get the revision from the flake
          # itself.
          git-rev = "${self.rev or self.dirtyRev or "Unknown"}";

          # The site is cross compiled using trunk and wasm. That's why we can build it with the
          # host toolchain, which already has support for wasm. However, the same is not true for
          # the target toolchain (aarch64-unknown-linux-musl), so we need to use a nix cross-compilation
          # strategy there
          site = pkgs.rustPlatform.buildRustPackage {
            pname = "vds-site";
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
              pushd vds-site
              trunk build --release --offline
              popd
              runHook postBuild
            ";

            installPhase = "
              runHook preInstall
              pushd vds-site
              mkdir -p $out
              cp -r dist $out/
              popd
              runHook postInstall
            ";
          };
        in
        targetPkgs.rustPlatform.buildRustPackage {
          pname = "vds-server";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [
            pkgs.lld
          ];

          buildPhase = "
            runHook preBuild
            pushd vds-server
            export VDS_SERVER_FRONTEND_PATH=${site}/dist
            export CARGO_TARGET_DIR=$(pwd)/../target
            export VDS_SERVER_NIX_GIT_REVISION=${git-rev}
            cargo build --release --offline -j $NIX_BUILD_CORES --target ${targetPkgs.stdenv.hostPlatform.rust.rustcTarget} --no-default-features
            popd
            runHook postBuild
          ";
        };
    in
    rec {
      packages = forEachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          targetPkgs = import nixpkgs {
            inherit system;
            crossSystem = {
              isStatic = true;
              config = "aarch64-unknown-linux-musl";
            };
          };
        in
        rec {
          # Local target
          vds = vdsPackageWithPkgs pkgs pkgs;

          # Cross compilation for RPi 4
          vds-target = vdsPackageWithPkgs pkgs targetPkgs;

          default = vds;
        }
      );

      # Much older versions of nix (from around 2022) used a different attribute to mark the default package.
      # This is included for compatibility with those versions.
      # See https://wiki.nixos.org/w/index.php?title=Flakes&oldid=7960#Output_schema
      defaultPackage = forEachSystem (system: packages."${system}".default);

      devShells = forEachSystem (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
        in
        rec {
          vds = pkgs.mkShell {
            nativeBuildInputs = [
              (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              pkgs.trunk
              pkgs.wasm-bindgen-cli_0_2_106
              pkgs.dart-sass
              pkgs.cargo-watch
              pkgs.cargo-deny
              pkgs.diesel-cli
              pkgs.bunyan-rs
            ];
          };

          default = vds;
        }
      );
    };
}
