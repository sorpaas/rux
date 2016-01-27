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
