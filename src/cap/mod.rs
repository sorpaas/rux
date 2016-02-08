use common::*;
use core::mem::size_of;
use core::marker::PhantomData;

mod frame;
mod untyped;
pub mod paging;
pub mod utils;
mod pool;
mod reserved;
mod vga;

use self::paging::{Frame};
use self::paging::{EntryFlags};
use self::paging::VirtualAddress;

pub use self::paging::{ActivePageTableCapability, InactivePageTableCapability};

//// A trait that represents all the capabilities.
pub trait Capability { }

/// A memory block capability represents a memory block.
pub trait MemoryBlockCapability {
    fn block_start_addr(&self) -> PhysicalAddress {
        self.block_end_addr() - self.block_size() + 1
    }

    fn block_size(&self) -> usize {
        self.block_end_addr() - self.block_start_addr() + 1
    }

    fn block_end_addr(&self) -> PhysicalAddress {
        self.block_start_addr() + self.block_size() - 1
    }
}

pub trait PageFrameCapability {
    type FrameIterator: Iterator<Item=Frame>;
    fn frames(&self) -> Self::FrameIterator;
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
    fn collect<F>(&mut self, mut f: F) where F: FnMut(T) -> Option<T>;
}

pub trait BorrowableCapability {
    type Borrowable;
    fn frame_start_addr(&self) -> PhysicalAddress;

    fn borrow<'r>(&self, virt: &VirtualAddress, table_cap: &'r ActivePageTableCapability) -> &'r Self::Borrowable {
        assert!(virt.table_addr() == table_cap.frame_start_addr());
        unsafe { &*(virt.addr() as *mut _) }
    }

    fn borrow_mut<'r>(&self, virt: &mut VirtualAddress, table_cap: &'r ActivePageTableCapability) -> &'r mut Self::Borrowable {
        assert!(virt.table_addr() == table_cap.frame_start_addr());
        unsafe { &mut *(virt.addr() as *mut _) }
    }
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
    flags: EntryFlags,
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
    guarded_frame_start_addr: Option<PhysicalAddress>,
    flags: EntryFlags,
}

pub struct VGABufferCapability;
