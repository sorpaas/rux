{ pkgs ? (
  let
    nixpkgs = import <nixpkgs>;
    pkgs_ = (nixpkgs {});
    rustOverlay = (pkgs_.fetchFromGitHub {
      owner = "mozilla";
      repo = "nixpkgs-mozilla";
      rev = "6179dd876578ca2931f864627598ede16ba6cdef";
      sha256 = "1lim10a674621zayz90nhwiynlakxry8fyz1x209g9bdm38zy3av";
    });
  in (nixpkgs {
    overlays = [
      (import (builtins.toPath "${rustOverlay}/rust-overlay.nix"))
      (self: super:
       with super;
       let nightly = lib.rustLib.fromManifest (lib.rustLib.manifest_v2_url {
                       channel = "nightly";
                       date = "2017-10-10";
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
  name = "target-spec";
  src = ./x86_64.json;
  phases = [ "buildPhase" ];
  buildPhase = ''
    mkdir -p $out
    cp $src $out/x86_64.json
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
    rustc --target=${x86_64-target-spec}/x86_64.json --out-dir=$out --crate-name=core --crate-type=lib ${rust.rust-src}/lib/rustlib/src/rust/src/libcore/lib.rs
  '';
};

libstd_unicode = stdenv.mkDerivation {
  name = "libstd_unicode";
  buildInputs = [
    rust.rustc
  ];
  phases = [ "buildPhase" ];
  buildPhase = ''
    mkdir -p $out
    rustc -L ${libcore} --target=${x86_64-target-spec}/x86_64.json --out-dir=$out --crate-name=std_unicode --crate-type=lib ${rust.rust-src}/lib/rustlib/src/rust/src/libstd_unicode/lib.rs
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
    rustc -L ${libcore} -L ${libstd_unicode} --target=${x86_64-target-spec}/x86_64.json --out-dir=$out --crate-name=alloc --crate-type=lib ${rust.rust-src}/lib/rustlib/src/rust/src/liballoc/lib.rs
  '';
};

triple = "x86_64-none-elf";

userspace-linker = stdenv.mkDerivation {
  name = "userspace-linker";
  phases = [ "buildPhase" ];
  buildPhase = ''
    mkdir -p $out
    cat <<EOT > $out/linker.ld
    ENTRY(start)
    OUTPUT_FORMAT(elf64-x86-64)
    EOT
  '';
};

in stdenv.mkDerivation {
  name = "rux-env";
  buildInputs = [
    gnumake
    (binutils.override { targetPlatform = { config = triple; }; })
    qemu
    file
    gdb
    rust.rustc
    rust.cargo
    curl
  ];

  ARCH = "x86_64";
  RUST_SRC = "${rust.rust-src}";
  TARGET_SPEC = "${x86_64-target-spec}/x86_64.json";
  USERSPACE_LINKER = "${userspace-linker}/linker.ld";

  LIBCORE = "${libcore}";
  LIBALLOC = "${liballoc}";
  LIBSTD_UNICODE = "${libstd_unicode}";

  LD = "${triple}-ld";
  AS = "${triple}-as";
  OBJDUMP = "${triple}-objdump";
  OBJCOPY = "${triple}-objcopy";
}
