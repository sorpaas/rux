{ pkgs ? import <nixpkgs> {} }:

with pkgs;
let funs = pkgs.callPackage ./nix/rust-nightly.nix { };
    cargoNightly = funs.cargo {
      date = "2016-08-11";
      hash = "0i4jj0cig2r10hicm8pblak3n7abgk1sql7krmhnbr4hw0m0r7rv";
    };

    rustNightly = funs.rust {
      date = "2016-08-11";
      hash = "169cibr9afpacqj6z60fxb2w28fwjf5zmywycrffs7vv1b43dpri";
    };

in stdenv.mkDerivation {
  name = "rux-build-env";

  SSL_CERT_FILE = "/etc/ssl/certs/ca-bundle.crt";

  buildInputs = [
    gnumake
    (binutils.override { cross = { config = "x86_64-none-elf"; }; })
    qemu
    file
    rustNightly
    cargoNightly
  ];
}
