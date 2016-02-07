use common::*;
use core::mem::size_of;
use core::marker::PhantomData;

mod frame;
mod untyped;
mod paging;
mod utils;
mod pool;
mod reserved;

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

trait PageFramePtr {
    fn get_frame_start_addr(&self) -> PhysicalAddress;
    fn set_frame_start_addr(&mut self, PhysicalAddress);
    fn get_frame_count(&self) -> usize;
    fn set_frame_count(&mut self, usize);
}

pub trait PageFrameCapability : PageFramePtr {
    fn frame_start_addr(&self) -> PhysicalAddress {
        self.get_frame_start_addr()
    }

    fn frame_count(&self) -> usize {
        self.get_frame_count()
    }

    fn frame_size(&self) -> usize {
        self.get_frame_count() * PAGE_SIZE
    }

    fn frame_end_addr(&self) -> PhysicalAddress {
        self.frame_start_addr() + self.frame_size() - 1
    }
}

pub struct CapabilityPool([Option<CapabilityUnion>; CAPABILITY_POOL_COUNT]);

pub enum CapabilityUnion {
    /// Memory resources capabilities, all has its start and end address, and a
    /// next pointer to the next region (if available).
    ///
    /// A memory resources capability is essentially a pointer to a memory
    /// location.

    Untyped(UntypedCapability),
    CapabilityPool(PageObjectCapability<CapabilityPool>),
    PageTable(PageTableCapability),
    KernelReservedBlock(KernelReservedBlockCapability),
    KernelReservedFrame(KernelReservedFrameCapability),
}

pub trait CapabilityMove<T: Capability> {
    fn put(&mut self, T);
    fn take_one(&mut self) -> Option<T>;
    fn select<F>(&mut self, f: F) -> Option<T> where F: Fn(&T) -> bool;
}

/// Untyped memory and page table are memory management tricks, those are not
/// actually accessible in the virtual memory.
impl Capability for UntypedCapability { }
pub struct UntypedCapability {
    block_start_addr: PhysicalAddress,
    block_size: usize,
}

/// Represent a page frame.
impl<T> Capability for PageObjectCapability<T> { }
pub struct PageObjectCapability<T> {
    block_start_addr: PhysicalAddress,
    block_size: usize,
    frame_start_addr: PhysicalAddress,
    frame_count: usize,
    _marker: PhantomData<T>,
}

/// Page table capability represents a P4 table.
impl Capability for PageTableCapability { }
pub struct PageTableCapability {
    block_start_addr: PhysicalAddress,
    block_size: usize,
    table_start_addr: PhysicalAddress,
}

impl Capability for KernelReservedBlockCapability { }
pub struct KernelReservedBlockCapability {
    block_start_addr: PhysicalAddress,
    block_size: usize,
}

impl Capability for KernelReservedFrameCapability { }
pub struct KernelReservedFrameCapability {
    block_start_addr: PhysicalAddress,
    block_size: usize,
    frame_start_addr: PhysicalAddress,
    frame_count: usize,
}
