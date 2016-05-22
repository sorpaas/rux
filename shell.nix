{ pkgs ? import <nixpkgs> {} }:

with pkgs;
let funs = pkgs.callPackage ./nix/rust-nightly.nix { };
    cargoNightly = funs.cargo {
      date = "2016-05-21";
      hash = "00b32hm8444dlxwl5v3v1mf4sw262n7yw04smsllr41kz2b8lq43";
    };

    rustNightly = funs.rust {
      date = "2016-05-21";
      hash = "0ylyq746hvqc8ibhi16vk7i12cnk0zlh46gr7s9g59dpx0j0f1nl";
    };

in stdenv.mkDerivation {
  name = "rux-build-env";

  SSL_CERT_FILE = "/etc/ssl/certs/ca-bundle.crt";

  buildInputs = [
    gnumake
    binutils-raw
    grub2
    nasm
    xorriso
    qemu
    rustNightly
    cargoNightly
  ];
}
