{
  description = "Low-Bandwidth Educational Access Platform";

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
      leapPackageWithPkgs =
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
            pname = "leap-site";
            version = "0.1.0";
            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
              trunk
              wasm-bindgen-cli_0_2_106
              dart-sass
              lld
            ];

            buildPhase = "
              runHook preBuild
              pushd leap-site
              trunk build --release --offline
              popd
              runHook postBuild
            ";

            installPhase = "
              runHook preInstall
              pushd leap-site
              mkdir -p $out
              cp -r dist $out/
              popd
              runHook postInstall
            ";
          };
        in
        targetPkgs.rustPlatform.buildRustPackage {
          pname = "leap-server";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            lld
          ];

          buildPhase = "
            runHook preBuild
            pushd leap-server
            export LEAP_SERVER_FRONTEND_PATH=${site}/dist
            export CARGO_TARGET_DIR=$(pwd)/../target
            export LEAP_SERVER_NIX_GIT_REVISION=${git-rev}
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
          leap = leapPackageWithPkgs pkgs pkgs;

          # Cross compilation for RPi 4
          leap-target = leapPackageWithPkgs pkgs targetPkgs;

          default = leap;
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
          leap = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              trunk
              wasm-bindgen-cli_0_2_106
              dart-sass
              cargo-watch
              cargo-deny
              diesel-cli
              bunyan-rs
            ];
          };

          leap-linux = (pkgs.buildFHSEnv {
            pname = "leap-linux";
            version = "1.0.0";

            targetPkgs = pkgs: with pkgs; [
              which
              gnused
              gnumake
              binutils
              diffutils
              gcc
              bash
              patch
              gzip
              bzip2
              perl
              gnutar
              cpio
              unzip
              rsync
              file
              bc
              findutils
              gawk

              wget

              # Additional host deps
              ncurses
              ncurses.dev
              pkg-config
              libxcrypt
              openssl
              openssl.dev
              gnutls
              gnutls.dev
              expat
              expat.dev
            ];
          }).env;

          default = leap;
        }
      );
    };
}
