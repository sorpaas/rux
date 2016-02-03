# Rux

This is a L4-like microKernel implementation in Rust. It uses a structure that
is similar to [seL4](https://github.com/seL4/seL4).

## Build

If you are using NixOS, simply `nix-shell` and `make run`.

If you are using other platforms, make sure that you have `nasm`, `grub2`,
`xorriso`, and `ld` installed. And then issue `make run`.
