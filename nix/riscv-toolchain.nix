{ stdenv, pkgs, fetchgit,
  arch, abi, newlibCflags ? "" }:

stdenv.mkDerivation rec {
    name = "riscv-toolchain";

    src = fetchgit {
      url = "https://github.com/riscv/riscv-gnu-toolchain";
      rev = "168ef95ba72bd79684df609ae3e05c6cd6499795";
      sha256 = "0rzs4xcc9laz9rsdgnfcxh46f752lc4ymxy1642ac3kh23693mwm";
    };

    buildInputs = with pkgs; [ autoconf automake texinfo
      gmp libmpc mpfr gawk bison flex texinfo gperf curl
      expat ];

    configureFlags = [
        "--with-arch=${arch}"
        "--with-abi=${abi}"
    ];

    CFLAGS_FOR_TARGET = newlibCflags;

    hardeningDisable = [ "all" ];
    dontPatchELF = true;
    dontStrip = true;
}
