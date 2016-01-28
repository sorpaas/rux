use common::*;
use core::ops::Drop;

use super::{MemoryBlockCapability};

/// Untyped memory and page table are memory management tricks, those are not
/// actually accessible in the virtual memory.

pub struct UntypedMemoryCapability {
    block_start_addr: PhysicalAddress,
    block_size: usize,
    parent_pool_cap_ptr: *mut CapabilityUnion,
    next_block_ptr: Option<*mut CapabilityUnion>,
    prev_block_ptr: Option<*mut CapabilityUnion>,
    first_child_block_ptr: Option<*mut CapabilityUnion>
    referred: bool,
}

impl MemoryBlockCapability for UntypedMemoryCapability {
    fn block_start_addr(&self) -> PhysicalAddress { self.block_start_addr }
    unsafe fn set_block_start_addr(&self, addr: PhysicalAddress) { self.block_start_addr = addr }

    fn block_size(&self) -> usize { self.physical_size }
    unsafe fn set_block_size(&self, size: usize) { self.block_size = size }

    unsafe fn next_block_ptr(&self) -> Option<*mut CapabilityUnion> { self.next_block_ptr }
    unsafe fn set_next_block_ptr(&self, ptr: Option<*mut CapabilityUnion>) { self.next_block_ptr = ptr }

    unsafe fn prev_block_ptr(&self) -> Option<*mut CapabilityUnion> { self.prev_block_ptr }
    unsafe fn set_prev_block_ptr(&self, ptr: Option<*mut CapabilityUnion>) { self.prev_block_ptr = ptr }
}

impl Capability for UntypedMemoryCapability {
    fn referred(&self) -> bool { self.referred }
    unsafe fn set_referred(&self, x: bool) { self.referred = x }

    unsafe fn parent_pool_cap_ptr(&self) -> *mut CapabilityUnion { self.parent_pool_cap_ptr }
    unsafe fn set_parent_pool_cap_ptr(&self, ptr: *mut CapabilityUnion) { self.parent_pool_cap_ptr = ptr }
}

impl UntypedMemoryCapability {
    pub unsafe fn retype_to_untyped(&mut self, block_size: usize)
                                    -> Option<UntypedMemoryCapability> {
        if (self.block_start_addr() + block_size >= self.block_end_addr()) {
            None
        } else {
            
        }
    }

    pub fn find_free_start_addr(&mut self, block_size: usize) -> Option<usize> {
        if self.
    }

    pub fn first_child_block(&mut self) -> Option<Referrer<CapabilityUnion>> {
        
    }

    pub fn from_untyped(cap: UntypedMemoryCapability, size: usize)
                        -> (UntypedMemoryCapability, Option<UntypedMemoryCapability>) {
        if cap.start_addr() + size + 1 >= cap.end_addr() {
            (cap, None)
        } else {
            let new_cap = UntypedMemoryCapability {
                start_addr: cap.start_addr(),
                size: size,
            };

            let cap = UntypedMemoryCapability::resize(cap, &new_cap);
            (new_cap, cap)
        }
    }

    pub fn resize<T: MemoryBlockCapability>(mut untyped: UntypedMemoryCapability, other: &T)
                                            -> Option<UntypedMemoryCapability> {
        assert!(untyped.physical_start_addr() == other.physical_start_addr(),
                "To resize, two capability's starting physical address must be the same.");
        assert!(untyped.physical_end_addr() >= other.physical_end_addr(),
                "To resize, the other capability must be within the untyped.");

        untyped.start_addr = other.physical_end_addr() + 1;
        untyped.size = untyped.size - other.physical_size();

        if untyped.size == 0 {
            None
        } else {
            Some(untyped)
        }
    }
}
