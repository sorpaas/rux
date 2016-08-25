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

    static kernel_stack_guard_page: u64;
    static kernel_end: u64;
}

use common::{PAddr, KERNEL_BASE};

pub fn kernel_end_address() -> PAddr {
    ((&kernel_end as *const _) as PAddr) - KERNEL_BASE
}

// TODO Change this to virtual address
pub fn kernel_stack_guard_page_address() -> PAddr {
    ((&kernel_stack_guard_page as *const _) as PAddr) - KERNEL_BASE
}
