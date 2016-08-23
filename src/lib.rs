#![feature(lang_items)]
#![feature(asm)]
#![no_std]

/// Macros, need to be loaded before everything else due to how rust parses
#[macro_use]
mod macros;

/// Achitecture-specific modules
#[cfg(target_arch="x86_64")] #[path="arch/x86_64/mod.rs"]
pub mod arch;

/// Exception handling (panic)
pub mod unwind;

/// Logging code
mod logging;

// Kernel entrypoint
#[lang="start"]
#[no_mangle]
pub fn kmain()
{
	log!("Hello world! 1={}", 1);
	loop {}
}
