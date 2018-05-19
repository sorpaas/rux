{ pkgs ? (
  let
    nixpkgs = import <nixpkgs>;
    pkgs_ = (nixpkgs {});
    rustOverlay = (pkgs_.fetchFromGitHub {
      owner = "mozilla";
      repo = "nixpkgs-mozilla";
      rev = "d7ba4e48037c0f944d01d7902fcdc8fa0766df24";
      sha256 = "033zk1pfnwh0ryrm1yzl9ybgqyhypgdxv1249a8z7cdy1rvb9zz4";
    });
  in (nixpkgs {
    overlays = [
      (import (builtins.toPath "${rustOverlay}/rust-overlay.nix"))
      (self: super:
       with super;
       let nightly = lib.rustLib.fromManifest (lib.rustLib.manifest_v2_url {
                       channel = "nightly";
                       date = "2018-05-17";
                     }) {
                       inherit (self) stdenv fetchurl patchelf;
                     };
           rustc_ = nightly.rustc;
           cargo_ = nightly.cargo;
           rust-src_ = nightly.rust-src;
           rust_ = nightly.rust;
       in {
        rust = {
          rustc = rustc_;
          cargo = cargo_;
          rust-src = rust-src_;
          rust = rust_;
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

libcompiler_builtins = stdenv.mkDerivation {
  name = "libcompiler_builtins";
  buildInputs = [
    rust.rustc
  ];
  phases = [ "buildPhase" ];
  buildPhase = ''
    mkdir -p $out
    rustc -L ${libcore} --cfg 'feature="compiler-builtins"' --target=${x86_64-target-spec}/x86_64.json --out-dir=$out --crate-name=compiler_builtins --crate-type=lib ${rust.rust-src}/lib/rustlib/src/rust/src/libcompiler_builtins/src/lib.rs
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
    rustc -L ${libcore} -L ${libcompiler_builtins} --target=${x86_64-target-spec}/x86_64.json --out-dir=$out --crate-name=alloc --crate-type=lib ${rust.rust-src}/lib/rustlib/src/rust/src/liballoc/lib.rs
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
    (binutils-unwrapped.override { targetPlatform = { config = triple; isiOS = false; isAarch64 = false; }; })
    qemu
    file
    gdb
    rust.rust
    rust.cargo
    curl
  ];

  ARCH = "x86_64";
  RUST_SRC = "${rust.rust-src}";
  TARGET_SPEC = "${x86_64-target-spec}/x86_64.json";
  USERSPACE_LINKER = "${userspace-linker}/linker.ld";

  LIBCORE = "${libcore}";
  LIBCOMPILER_BUILTINS = "${libcompiler_builtins}";
  LIBALLOC = "${liballoc}";

  LD = "${triple}-ld";
  AS = "${triple}-as";
  OBJDUMP = "${triple}-objdump";
  OBJCOPY = "${triple}-objcopy";
}
