{ pkgs ? import <nixpkgs> {} }:

with pkgs;
let funs = pkgs.callPackage ./nix/rust-nightly.nix { };
    cargoNightly = funs.cargo {
      date = "2016-10-04";
      hash = "1na7achkhl0vjlr0ncfkhvl0pqzbkhn02b6n1d7z1acr00wrbg75";
    };

    rustNightly = funs.rust {
      date = "2016-10-04";
      hash = "1bgwncmmxwdvsh59q7fwv3rdbb9l5bvlaj0fyr700y4rb7bxrs93";
    };

in stdenv.mkDerivation {
  name = "rux-build-env";

  SSL_CERT_FILE = "/etc/ssl/certs/ca-bundle.crt";

  buildInputs = [
    gnumake
    (binutils.override { cross = { config = "x86_64-none-elf"; }; })
    qemu
    file
    gdb
    rustNightly
    cargoNightly
    curl
  ];
}
