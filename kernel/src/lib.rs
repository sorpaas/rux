#![feature(lang_items)]
#![feature(panic_implementation)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]

#![no_std]

extern crate rlibc;

#[macro_use]
mod macros;
#[cfg(target_arch="riscv32")] #[path="arch/riscv32/mod.rs"]
#[macro_use]
pub mod arch;
mod logging;
