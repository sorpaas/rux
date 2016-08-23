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
    
    let hello = b"Hello World!";
    let color_byte = 0x1f; // white foreground, blue background

    let mut hello_colored = [color_byte; 24];
    for (i, char_byte) in hello.into_iter().enumerate() {
        hello_colored[i*2] = *char_byte;
    }

    // write `Hello World!` to the center of the VGA text buffer
    let buffer_ptr = (0xFFFFFFFF80000000 + 0xb8000 as u64 + 1988) as *mut _;
    unsafe { *buffer_ptr = hello_colored };
    
	loop {}
}
