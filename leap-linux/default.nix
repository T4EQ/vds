{ inputs, ... }:
let
  buildroot-nix = inputs.buildroot-nix;
  depsForBuildroot =
    pkgs: with pkgs; [
      git

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
      pkg-config
      libxcrypt
      ncurses
      ncurses.dev
      openssl
      openssl.dev
      gnutls
      gnutls.dev
      expat
      expat.dev

    ];
in
{
  perSystem =
    { config, pkgs, ... }:
    {
      packages =
        let
          leapBuildrootSrc = pkgs.stdenv.mkDerivation {
            name = "leap-linux-src";
            src = ./.;

            buildPhase = ''
              echo "done"
            '';

            installPhase = ''
              mkdir -p $out/
              cp -r ./* $out/

              mkdir -p $out/package/leap/src
              cp ${config.packages.leap-target}/bin/leap-server $out/package/leap/src/leap-server
            '';

          };

          buildrootPackages = buildroot-nix.lib.mkBuildroot {
            name = "leap-linux";
            inherit pkgs;

            defconfig = "leap_defconfig";
            lockfile = ./buildroot.lock;

            src = pkgs.fetchgit {
              url = "https://gitlab.com/buildroot.org/buildroot";
              branchName = "2026.02";
              sha256 = "sha256-XKDo55apHycjsMNUCAYKBqWx6emDNGKU3kJcYUcywh0=";
            };

            externalSrc = leapBuildrootSrc;
            patchSrc = ./patches;
            nativeBuildInputs = depsForBuildroot pkgs;
          };
        in
        {
          leap-linux-lockfile = buildrootPackages.packageLockFile;
          leap-linux = buildrootPackages.buildroot.overrideAttrs (old: {
            # Buildroot's openssh.mk installs ssh-keysign directly with mode 4711
            # (setuid), bypassing Makefile.in. Patch the .mk file so the Nix
            # sandbox (which blocks SUID chmod) doesn't fail.
            patchPhase = old.patchPhase + ''
              sed -i 's/-m 4711/-m 0711/' package/openssh/openssh.mk
            '';
          });
        };

      devShells = {
        leap-linux =
          (pkgs.buildFHSEnv {
            pname = "leap-linux";
            version = "1.0.0";
            targetPkgs = depsForBuildroot;
          }).env;
      };
    };
}
