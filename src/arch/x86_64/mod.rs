/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang) 
 *
 * arch/amd64/mod.rs
 * - Top-level file for amd64 architecture
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */

// x86 port IO 
#[path = "io.rs"]
mod x86_io;

// Debug output channel (uses serial)
#[path = "debug.rs"]
pub mod debug;

extern {
    pub static multiboot_sig: u32;
    pub static multiboot_ptr: u64;

    static init_pml4: u64;
    static init_stack_base: u64;
    static init_stack: u64;
    static kernel_end: u64;
}

use common::{PAddr, KERNEL_BASE};

pub fn kernel_end_address() -> PAddr {
    ((&kernel_end as *const _) as PAddr) - KERNEL_BASE
}

pub fn kernel_stack_address() -> PAddr {
    ((&init_stack as *const _) as PAddr) - KERNEL_BASE
}

pub fn kernel_stack_init_base_address() -> PAddr {
    ((&init_stack_base as *const _) as PAddr) - KERNEL_BASE
}

pub fn kernel_stack_max_base_address() -> PAddr {
    ((&init_pml4 as *const _) as PAddr) - KERNEL_BASE
}
