let

moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };

in with nixpkgs;

let

rustChannel = rustChannelOf { date = "2018-08-04"; channel = "nightly"; };
rustc = rustChannel.rustc;
rust-src = "${rustChannel.rust-src}/lib/rustlib/src/rust/src";

target = "riscv32imac-unknown-none-elf";

libcore = stdenv.mkDerivation {
  name = "libcore";
  buildInputs = [
    rustc
  ];
  phases = [ "buildPhase" ];
  buildPhase = ''
    mkdir -p $out
    rustc --target=${target} --out-dir=$out --crate-name=core --crate-type=lib ${rust-src}/libcore/lib.rs
  '';
};

libcompiler_builtins = stdenv.mkDerivation {
  name = "libcompiler_builtins";
  buildInputs = [
    rustc
  ];
  phases = [ "buildPhase" ];
  buildPhase = ''
    mkdir -p $out
    rustc -L ${libcore} --cfg 'feature="compiler-builtins"' --target=${target} --out-dir=$out --crate-name=compiler_builtins --crate-type=lib ${rust-src}/libcompiler_builtins/src/lib.rs
  '';
};

liballoc = stdenv.mkDerivation {
  name = "liballoc";
  buildInputs = [
    rustc
  ];
  phases = [ "buildPhase" ];
  buildPhase = ''
    mkdir -p $out
    rustc -L ${libcore} -L ${libcompiler_builtins} --target=${target} --out-dir=$out --crate-name=alloc --crate-type=lib ${rust-src}/liballoc/lib.rs
  '';
};

in

stdenv.mkDerivation {
  name = "rux-shell";
  buildInputs = [
    rustc
  ];

  RUST_SRC = "${rust-src}";

  LIBCORE = "${libcore}";
  LIBCOMPILER_BUILTINS = "${libcompiler_builtins}";
  LIBALLOC = "${liballoc}";
}
