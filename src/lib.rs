#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(unique)]
#![no_std]

extern crate x86;
extern crate spin;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitflags;

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
mod utils;
mod common;

use core::mem;
use core::slice;
use common::{PAddr, VAddr};
use arch::{ArchInfo, MemoryRegion};

#[no_mangle]
pub fn kmain(archinfo: ArchInfo)
{
    // let hello = b"Hello World!";
    // let color_byte = 0x1f; // white foreground, blue background

    // let mut hello_colored = [color_byte; 24];
    // for (i, char_byte) in hello.into_iter().enumerate() {
    //     hello_colored[i*2] = *char_byte;
    // }

    // // write `Hello World!` to the center of the VGA text buffer
    // let buffer_ptr = (0xFFFFFFFF80000000 + 0xb8000 as u64 + 1988) as *mut _;
    // unsafe { *buffer_ptr = hello_colored };

    use arch::{object_pool_pt, object_pool_pt_mut,
               with_object, with_object_mut,
               kernel_pd_paddr, kernel_pml4_paddr};
    use arch::paging::{PD, PML4};

    with_object(kernel_pml4_paddr(), |pml4: &PML4| {
        for area in pml4.iter() {
            log!("{:?}", area.get_address());
        }
    });

    for region in archinfo.memory_regions() {
        log!("{:?}", region);
    }

    log!("hello, world!");
    
	loop {}
}
