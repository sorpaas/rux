use common::*;
use alloc::boxed::Box;
use paging::table::PageTable;
use paging::table::{PageTableLevel,
                    PageTableLevel4, PageTableLevel3,
                    PageTableLevel2, PageTableLevel1};
use core::mem::size_of;

use self::pool::CapabilityPool;
use self::untyped::UntypedMemoryCapability;

pub mod pool;
pub mod untyped;

pub trait MemoryBlockCapability {
    fn start_addr(&self) -> PhysicalAddress;
    fn size(&self) -> usize;

    fn physical_start_addr(&self) -> PhysicalAddress;
    fn physical_size(&self) -> usize;
}

impl<T> MemoryBlockCapability for T {
    pub fn end_addr(&self) -> PhysicalAddress {
        self.start_addr() + self.size()
    }

    pub fn physical_end_addr(&self) -> PhysicalAddress {
        self.physical_start_addr() + self.physical_size()
    }
}

// // You can only access active page table using the recursive trick. Other page
// // tables need to be temporarily mapped in order to be accessible.

// pub struct PageTableCapability {
//     start: PhysicalAddress,
//     next: Option<Box<CapabilityUnion>>,
// }

// // Kernel page are created by retype untyped memory.

// pub struct KernelPageCapability {
//     start: PhysicalAddress,
//     count: usize,
//     mapped: bool,
//     next: Option<Box<CapabilityUnion>>,
// }

// // VGA buffer is a pre-created capability in the main kernel capability pool.

// pub struct VGABufferCapability {
//     start: PhysicalAddress,
//     size: usize,
// }

