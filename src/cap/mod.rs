use common::*;
use core::mem::size_of;

mod pool;
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

/// Page block pointer.
trait PageBlockPtr {
    fn get_page_start_addr(&self) -> PhysicalAddress;
    fn set_page_start_addr(&mut self, PhysicalAddress);
    fn get_page_counts(&self) -> usize;
    fn set_page_counts(&mut self, usize);
}

/// Page block capability.
pub trait PageBlockCapability<T> : PageBlockPtr {
    fn page_start_addr(&self) -> PhysicalAddress {
        self.get_page_start_addr()
    }

    fn page_counts(&self) -> usize {
        self.get_page_counts()
    }

    fn page_size(&self) -> usize {
        self.get_page_counts() * PAGE_SIZE
    }

    fn page_end_addr(&self) -> PhysicalAddress {
        self.page_start_addr() + self.page_size() - 1
    }

    fn object_size() -> usize {
        size_of::<T>()
    }
}

pub enum CapabilityUnion {
    /// Memory resources capabilities, all has its start and end address, and a
    /// next pointer to the next region (if available).
    ///
    /// A memory resources capability is essentially a pointer to a memory
    /// location.

    UntypedMemory(UntypedCapability),
    CapabilityPool(CapabilityPoolCapability),
    PageTable(PageTableCapability),
}

/// Untyped memory and page table are memory management tricks, those are not
/// actually accessible in the virtual memory.
pub struct UntypedCapability {
    block_start_addr: PhysicalAddress,
    block_size: usize,
}

/// The main kernel capability pool is static. Other capability pools are created
/// by retype kernel page.
pub struct CapabilityPoolCapability {
    block_start_addr: PhysicalAddress,
    block_size: usize,
    page_start_addr: PhysicalAddress,
    page_counts: usize,
}

/// Page table capability represents a P4 table.
pub struct PageTableCapability {
    block_start_addr: PhysicalAddress,
    block_size: usize,
    table_start_addr: PhysicalAddress,
}
