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

mod paging;
mod init;
mod addr;
mod interrupt;
mod segmentation;

pub mod cap;
const KERNEL_BASE: u64 = 0xFFFFFFFF80000000;

extern {
    static kernel_end: u64;
}

fn kernel_start_paddr() -> PAddr {
    PAddr::from(0x100000: usize)
}

fn kernel_start_vaddr() -> VAddr {
    unsafe { kernel_paddr_to_vaddr(kernel_start_paddr()) }
}

fn kernel_end_paddr() -> PAddr {
    PAddr::from((&kernel_end as *const _) as u64 - KERNEL_BASE)
}

fn kernel_end_vaddr() -> VAddr {
    unsafe { kernel_paddr_to_vaddr(kernel_end_paddr()) }
}

unsafe fn kernel_paddr_to_vaddr(addr: PAddr) -> VAddr {
    VAddr::from(addr.into(): u64 + KERNEL_BASE)
}


// Public interfaces
pub use self::paging::{MemoryObject};
pub use self::interrupt::{enable_interrupt, disable_interrupt, set_interrupt_handler,
                          InterruptInfo};
pub use self::init::{InitInfo};
// pub use self::cap::{ArchCap, PageHalf, PageFull};
pub use self::addr::{PAddr, VAddr};

// pub type TopPageTableHalf = self::cap::PML4Half;
// pub type TopPageTableFull = self::cap::PML4Full;
