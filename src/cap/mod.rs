use common::*;
use core::mem::size_of;
use core::marker::PhantomData;

mod frame;
mod untyped;
mod paging;
mod utils;

//// A trait that represents all the capabilities.
pub trait Capability { }

/// Internal use that represents a memory block pointer, which is used to
/// implement a memory block capability.
trait MemoryBlockPtr {
    fn get_block_start_addr(&self) -> PhysicalAddress;
    fn set_block_start_addr(&mut self, PhysicalAddress);
    fn get_block_size(&self) -> usize;
    fn set_block_size(&mut self, usize);
}

/// A memory block capability represents a memory block.
pub trait MemoryBlockCapability : MemoryBlockPtr {
    fn block_start_addr(&self) -> PhysicalAddress {
        self.get_block_start_addr()
    }

    fn block_size(&self) -> usize {
        self.get_block_size()
    }

    fn block_end_addr(&self) -> PhysicalAddress {
        self.block_start_addr() + self.block_size() - 1
    }
}

pub struct CapabilityPool([CapabilityUnion; CAPABILITY_POOL_COUNT]);

pub enum CapabilityUnion {
    /// Memory resources capabilities, all has its start and end address, and a
    /// next pointer to the next region (if available).
    ///
    /// A memory resources capability is essentially a pointer to a memory
    /// location.

    UntypedMemory(UntypedCapability),
    CapabilityPool(PageFrameCapability<CapabilityPool>),
    PageTable(PageTableCapability),
}

/// Untyped memory and page table are memory management tricks, those are not
/// actually accessible in the virtual memory.
pub struct UntypedCapability {
    block_start_addr: PhysicalAddress,
    block_size: usize,
}

/// Represent a page frame.
pub struct PageFrameCapability<T> {
    block_start_addr: PhysicalAddress,
    block_size: usize,
    frame_start_addr: PhysicalAddress,
    frame_counts: usize,
    _marker: PhantomData<T>,
}

/// Page table capability represents a P4 table.
pub struct PageTableCapability {
    block_start_addr: PhysicalAddress,
    block_size: usize,
    table_start_addr: PhysicalAddress,
}
