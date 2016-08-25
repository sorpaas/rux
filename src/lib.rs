#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![no_std]

extern crate x86;

#[macro_use]
extern crate lazy_static;

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

use arch::{multiboot_sig, multiboot_ptr};

// Kernel entrypoint
#[lang="start"]
#[no_mangle]
pub fn kmain()
{
    assert!(multiboot_sig == 0x2badb002);
    
    log!("multiboot_sig: 0x{:x}", multiboot_sig);
    log!("multiboot_ptr: 0x{:x}", multiboot_ptr);

    log!("kernel_stack: 0x{:x}", arch::kernel_stack_address());
    log!("kernel_stack_init_base: 0x{:x}", arch::kernel_stack_init_base_address());
    log!("kernel_stack_max_base: 0x{:x}", arch::kernel_stack_max_base_address());
    log!("kernel_end: 0x{:x}", arch::kernel_end_address());
    
    let bootinfo = unsafe {
        multiboot::Multiboot::new(multiboot_ptr as u64, |addr, size| {
            let ptr = mem::transmute(addr + KERNEL_BASE);
            Some(slice::from_raw_parts(ptr, size))
        })
    }.unwrap();

    log!("Bootinfo: {:?}", bootinfo);

    log!("Lower memory bound: 0x{:x}", bootinfo.lower_memory_bound().unwrap());
    log!("Upper memory bound: 0x{:x}", bootinfo.upper_memory_bound().unwrap());

    log!("Memory regions:");
    for area in bootinfo.memory_regions().unwrap() {
        log!("    Base: 0x{:x}, length: 0x{:x}, type: {:?}", area.base_address(), area.length(), area.memory_type());
    }
    
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
