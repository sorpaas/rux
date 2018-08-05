#![feature(lang_items)]
#![feature(panic_implementation)]
#![feature(asm)]

#![no_std]

#[cfg(target_arch="riscv32")] #[path="arch/riscv32/mod.rs"]
#[macro_use]
pub mod arch;
