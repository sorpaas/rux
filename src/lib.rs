#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![no_std]

extern crate x86;

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

mod common;
mod multiboot;

use core::mem;
use core::slice;
use common::KERNEL_BASE;

// Kernel entrypoint
#[lang="start"]
#[no_mangle]
pub fn kmain(multiboot_addr: u64)
{
    log!("Multiboot addr: 0x{:x}", multiboot_addr);
    let bootinfo = unsafe {
        multiboot::Multiboot::new(multiboot_addr, |addr, size| {
            let ptr = mem::transmute(addr + KERNEL_BASE);
            Some(slice::from_raw_parts(ptr, size))
        })
    };
    
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
