{ pkgs ? import <nixpkgs> {} }:

with pkgs;

stdenv.mkDerivation {
  name = "rux-build-env";

  SSL_CERT_FILE = "/etc/ssl/certs/ca-bundle.crt";

  buildInputs = [
    gnumake
    binutils-raw
    grub2
    nasm
    xorriso
    qemu
    rustUnstable.rustc
    rustUnstable.cargo
  ];
}
