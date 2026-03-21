{ ... }:
{
  perSystem =
    { pkgs, ... }:
    {
      devShells = {
        leap-linux =
          (pkgs.buildFHSEnv {
            pname = "leap-linux";
            version = "1.0.0";

            targetPkgs =
              pkgs: with pkgs; [
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
                ncurses.dev
                openssl.dev
                gnutls.dev
                expat.dev
              ];
          }).env;
      };
    };
}
