{ pkgs ? (
  let
    nixpkgs = import <nixpkgs>;
    pkgs_ = (nixpkgs {});
    rustOverlay = (pkgs_.fetchFromGitHub {
      owner = "mozilla";
      repo = "nixpkgs-mozilla";
      rev = "e2a920faec5a9ebd6ff34abf072aacb4e0ed6f70";
      sha256 = "1lq7zg388y4wrbl165wraji9dmlb8rkjaiam9bq28n3ynsp4b6fz";
    });
  in (nixpkgs {
    overlays = [
      (import (builtins.toPath "${rustOverlay}/rust-overlay.nix"))
      (self: super:
       with super;
       let nightly = lib.rustLib.fromManifest (lib.rustLib.manifest_v2_url {
                       channel = "nightly";
                       date = "2017-03-21";
                     }) {
                       inherit (self) stdenv fetchurl patchelf;
                     };
           rustc_ = nightly.rustc;
           cargo_ = nightly.cargo;
           rust-src_ = nightly.rust-src;
       in {
        rust = {
          rustc = rustc_;
          cargo = cargo_;
          rust-src = rust-src_;
        };
      })
    ];
  }))
}:

with pkgs;

let

x86_64-target-spec = stdenv.mkDerivation {
  name = "target-spec.json";
  src = ./x86_64.json;
  phases = [ "buildPhase" ];
  buildPhase = ''
    cp $src $out
  '';
};

libcore = stdenv.mkDerivation {
  name = "libcore";
  buildInputs = [
    rust.rustc
  ];
  phases = [ "buildPhase" ];
  buildPhase = ''
    mkdir -p $out
    rustc --target=${x86_64-target-spec} --out-dir=$out --crate-type=lib ${rust.rust-src}/lib/rustlib/src/rust/src/libcore/lib.rs
  '';
};

liballoc = stdenv.mkDerivation {
  name = "liballoc";
  buildInputs = [
    rust.rustc
  ];
  phases = [ "buildPhase" ];
  buildPhase = ''
    mkdir -p $out
    rustc -L ${libcore} --target=${x86_64-target-spec} --out-dir=$out --crate-type=lib ${rust.rust-src}/lib/rustlib/src/rust/src/liballoc/lib.rs
  '';
};

triple = "x86_64-none-elf";

userspace-linker = stdenv.mkDerivation {
  name = "userspace-linker.ld";
  phases = [ "buildPhase" ];
  buildPhase = ''
    cat <<EOT > $out
    ENTRY(start)
    OUTPUT_FORMAT(elf64-x86-64)
    EOT
  '';
};

in stdenv.mkDerivation {
  name = "rux-env";
  buildInputs = [
    gnumake
    (binutils.override { cross = { config = triple; }; })
    qemu
    file
    gdb
    rust.rustc
    rust.cargo
    curl
  ];

  ARCH = "x86_64";
  RUST_SRC = "${rust.rust-src}";
  TARGET_SPEC = "${x86_64-target-spec}";
  USERSPACE_LINKER = "${userspace-linker}";

  LIBCORE = "${libcore}";
  LIBALLOC = "${liballoc}";

  LD = "${triple}-ld";
  AS = "${triple}-as";
  OBJDUMP = "${triple}-objdump";
  OBJCOPY = "${triple}-objcopy";
}
