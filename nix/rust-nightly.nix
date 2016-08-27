{ stdenv, lib, buildEnv, makeWrapper, runCommand, fetchzip, zlib, rsync }:

# rustc and cargo nightly binaries

let
  mkTarget = system:
    if      system == "i686-linux"    then "i686-unknown-linux-gnu"
    else if system == "x86_64-linux"  then "x86_64-unknown-linux-gnu"
    else if system == "i686-darwin"   then "i686-apple-darwin"
    else if system == "x86_64-darwin" then "x86_64-apple-darwin"
    else abort "no snapshot to bootstrap for this platform (missing target triple)";

  mkUrl = { pname, archive, date, system }:
    "${archive}/${date}/${pname}-nightly-${mkTarget system}.tar.gz";

  generic = { pname, archive, exes }:
      { date, hash, system ? stdenv.system }:
      stdenv.mkDerivation rec {
    name = "${pname}-${version}";
    version = "nightly-${date}";
    # TODO meta;

    src = fetchzip {
      url = mkUrl { inherit pname archive date system; };
      sha256 = hash;
    };

    nativeBuildInputs = [ rsync ];

    dontStrip = true;

    unpackPhase = ""; # skip it

    installPhase = ''
      rsync --chmod=u+w -r $src/*/ $out/
    '';

    preFixup = if stdenv.isLinux then let
      # it's overkill, but fixup will prune
      rpath = "$out/lib:" + lib.makeLibraryPath [ zlib stdenv.cc.cc.lib ];
    in ''
      for executable in ${lib.concatStringsSep " " exes}; do
        patchelf \
          --interpreter "$(< $NIX_CC/nix-support/dynamic-linker)" \
          --set-rpath "${rpath}" \
          "$out/bin/$executable"
      done
      for library in $out/lib/*.so; do
        patchelf --set-rpath "${rpath}" "$library"
      done
    '' else "";
  };

in rec {
  rustc = generic {
    pname = "rustc";
    archive = "https://static.rust-lang.org/dist";
    exes = [ "rustc" "rustdoc" ];
  };

  rustcWithSysroots = { rustc, sysroots ? [] }: buildEnv {
    name = "combined-sysroots";
    paths = [ rustc ] ++ sysroots;
    pathsToLink = [ "/lib" "/share" ];
    #buildInputs = [ makeWrapper ];
    # Can't use wrapper script because of https://github.com/rust-lang/rust/issues/31943
    postBuild = ''
      mkdir -p $out/bin/
      cp ${rustc}/bin/* $out/bin/
    '';
  };

  rust-std = { date, hash, system ? stdenv.system }: stdenv.mkDerivation rec {
    # Strip install.sh, etc
    pname = "rust-std";
    version = "nightly-${date}";
    name = "${pname}-${version}-${system}";
    src = fetchzip {
      url = mkUrl {
        archive = "https://static.rust-lang.org/dist";
        inherit pname date system;
      };
      sha256 = hash;
    };
    buildCommand= ''
      mkdir -p $out
      cp -r "$src"/*/* $out/
      rm $out/manifest.in
    '';
  };

  cargo = generic {
    pname = "cargo";
    archive = "https://static.rust-lang.org/cargo-dist";
    exes = [ "cargo" ];
  };

  rust = generic {
    pname = "rust";
    archive = "https://static.rust-lang.org/dist";
    exes = [ "rustc" "rustdoc" "cargo" ];
  };
}
