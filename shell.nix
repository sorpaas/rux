{ pkgs ? import <nixpkgs> {} }:

with pkgs;

stdenv.mkDerivation {
  name = "rux-build-env";

  buildInputs = [
    gnumake
    binutils
    grub2
    nasm
    xorriso
  ];
}
