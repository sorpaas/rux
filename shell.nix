{ pkgs ? import <nixpkgs> {} }:

with pkgs;
let funs = pkgs.callPackage ./nix/rust-nightly.nix { };
    cargoNightly = funs.cargo {
      date = "2016-09-07";
      hash = "0wy8l6n9rlcpk4kwdqy55xi53q5q86n4n9z20kvap0lly60mxmb9";
    };

    rustNightly = funs.rust {
      date = "2016-09-07";
      hash = "1p9xbja98cpflq0x7wkiqjji7mwpayaf20jxiy9k2frqn2329876";
    };

in stdenv.mkDerivation {
  name = "rux-build-env";

  SSL_CERT_FILE = "/etc/ssl/certs/ca-bundle.crt";

  buildInputs = [
    gnumake
    (binutils.override { cross = { config = "x86_64-none-elf"; }; })
    nasm
    qemu
    file
    gdb
    rustNightly
    cargoNightly
  ];
}
