{ supportedSystems ? ["x86_64-linux"] }:

with import <nixpkgs/pkgs/top-level/release-lib.nix> { inherit supportedSystems; };
{
  build = testOn supportedSystems (pkgs:
    let funs = pkgs.callPackage ./nix/rust-nightly.nix { };
        rustNightly = funs.rust {
          date = "2016-05-21";
          hash = "0ylyq746hvqc8ibhi16vk7i12cnk0zlh46gr7s9g59dpx0j0f1nl";
        };

        cargoNightly = pkgs.rustPlatform.cargo // { rustc = rustNightly; };

        buildRustPackage = pkgs.callPackage <nixpkgs/pkgs/build-support/rust> {
          cargo = cargoNightly;
          rustRegistry = pkgs.callPackage <nixpkgs/pkgs/top-level/rust-packages.nix> { };
        };

    # See https://github.com/NixOS/nixpkgs/blob/master/pkgs/build-support/rust/default.nix
    in buildRustPackage rec {
      name = "rux-${version}";
      version = "0.0.1";
      src = ./.;
      buildInputs = with pkgs; [
        gnumake
        binutils-raw
        grub2
        nasm
        xorriso
        qemu
        rustNightly
        cargoNightly
      ];
      depsSha256 = "1rhzkvlan9mv75jfbynpcdyg76ldid0hnbrfdmra4rl3fs221w08";
      buildPhase = ''
        cargo --version
        make iso
      '';
      installPhase = ''
        mkdir -p $out/bin
        cp -r build/* $out
      '';
      doCheck = false;
    });
}
