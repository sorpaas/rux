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
pub mod paging;

mod init;
mod addr;

/// length: memory length (usize)
/// page_size: page size (usize)
/// vaddr, paddr

pub use self::addr::{PAddr, VAddr};
pub use self::paging::{PD, BASE_PAGE_LENGTH, LARGE_PAGE_LENGTH,
                       with_object, with_object_mut};
pub use self::init::{ArchInfo, MemoryRegion, object_pool_pt, object_pool_pt_mut,
                     kernel_pml4_paddr, kernel_pd_paddr, kernel_pdpt_paddr,
                     OBJECT_POOL_START_VADDR,
                     OBJECT_POOL_SIZE, OBJECT_POOL_PT_VADDR};
pub const KERNEL_BASE: u64 = 0xFFFFFFFF80000000;

extern {
    static mut init_pd: PD;
    static kernel_end: u64;
}

pub fn kernel_start_paddr() -> PAddr {
    PAddr::from_u64(0x100000)
}

pub fn kernel_start_vaddr() -> VAddr {
    unsafe { kernel_paddr_to_vaddr(kernel_start_paddr()) }
}

pub fn kernel_end_paddr() -> PAddr {
    PAddr::from_u64((&kernel_end as *const _) as u64 - KERNEL_BASE)
}

pub fn kernel_end_vaddr() -> VAddr {
    unsafe { kernel_paddr_to_vaddr(kernel_end_paddr()) }
}

pub unsafe fn kernel_paddr_to_vaddr(addr: PAddr) -> VAddr {
    VAddr::from_u64(addr.as_u64() + KERNEL_BASE)
}
