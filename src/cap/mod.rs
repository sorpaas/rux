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
    fn block_start_addr(&self) -> PhysicalAddress;
    unsafe fn set_block_start_addr(&mut self, PhysicalAddress);

    fn block_size(&self) -> usize;
    unsafe fn set_block_size(&mut self, usize);

    unsafe fn next_block_ptr(&self) -> Option<*mut CapabilityUnion>;
    unsafe fn set_next_block_ptr(&mut self, Option<*mut CapabilityUnion>);

    unsafe fn prev_block_ptr(&self) -> Option<*mut CapabilityUnion>;
    unsafe fn set_prev_block_ptr(&mut self, Option<*mut CapabilityUnion>);

    fn next_block(&self) -> Option<Referrer<CapabilityUnion>> {
        unsafe { self.next_block_ptr()
                 .and_then(|ptr| Referrer<CapabilityUnion>::new(ptr)) }
    }

    fn prev_block(&self) -> Option<Referrer<CapabilityUnion>> {
        unsafe { self.prev_block_ptr()
                 .and_then(|ptr| Referrer<CapabilityUnion>::new(ptr)) }
    }
}

pub trait Capability {
    unsafe fn parent_pool_cap_ptr(&self) -> *mut CapabilityPoolCapability;
    unsafe fn set_parent_pool_cap_ptr(&mut self, *mut CapabilityPoolCapability);

    fn referred(&self) -> bool;
    unsafe fn set_referred(&mut self);

    fn refer(&mut self) {
        assert!(!self.referred(), "Already referred to this object.");
        unsafe { self.set_referred(true) }
    }

    fn unrefer(&mut self) {
        assert!(self.referred(), "This object is not referred anywhere.");
        unsafe { self.set_referred(false) }
    }

    fn parent_pool_cap(&self) -> Referrer<CapabilityUnion> {
        unsafe {
            Referrer<CapabilityUnion>::new(self.parent_pool_cap_ptr())
        };
    }
}

pub struct Referrer<T: Capability> {
    ptr: NonZero<*mut T>,
    _marker: PhantomData<T>,
}

impl<T: Capability> Referrer<T> {
    pub const unsafe fn new(ptr: *mut T) -> Referrer<T> {
        let unique = Referrer { ptr: NonZero::new(ptr),
                                _marker: PhantomData<T>, };
        unsafe { unique.borrow_mut().refer(); }
        unique
    }

    pub fn borrow<'r>(&'r self) -> &'r T {
        unsafe { &*self.ptr }
    }

    pub fn borrow_mut<'r>(&'r mut self) -> &'r mut T {
        unsafe { &mut *self.ptr }
    }
}

#[unsafe_destructur]
impl<T: Capability> Drop for Referrer<T> {
    fn drop(&mut self) {
        unsafe { self.borrow_mut().unrefer() }
    }
}

pub enum CapabilityUnion {
    /// Memory resources capabilities, all has its start and end address, and a
    /// next pointer to the next region (if available).
    ///
    /// A memory resources capability is essentially a pointer to a memory
    /// location.

    UntypedMemory(UntypedMemoryCapability),
    CapabilityPool(CapabilityPoolCapability),
}

impl CapabilityUnion {
    pub fn as_untyped_memory(cap: CapabilityUnion) -> Option<UntypedMemoryCapability> {
        if let CapabilityUnion::UntypedMemory(x) = cap
        { Some(x) } else { None }
    }

    pub fn as_capability_pool(cap: CapabilityUnion) -> Option<CapabilityPoolCapability> {
        if let CapabilityUnion::CapabilityPool(x) = cap
        { Some(x) } else { None }
    }
}
