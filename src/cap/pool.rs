use common::*;
use core::mem::{align_of, replace, uninitialized, size_of};
use core::ops::Drop;
use core::ptr;
use alloc::boxed::Box;

use super::MemoryBlockCapability;
use super::untyped::UntypedMemoryCapability;

pub struct CapabilityPool([Option<CapabilityUnion>; CAPABILITY_POOL_COUNT]);

// The main kernel capability pool is static. Other capability pools are created
// by retype kernel page.

pub struct CapabilityPoolCapability {
    memory_start_addr: PhysicalAddress,
    object_start_addr: PhysicalAddress,
    parent_pool_cap_ptr: Option<*const CapabilityPoolCapability>,
    next_ptr: Option<*const CapabilityUnion>,
    prev_ptr: Option<*const CapabilityUnion>,
    referred: bool,
}

impl MemoryBlockCapability for CapabilityPoolCapability {
    fn memory_start_addr(&self) -> PhysicalAddress {
        self.memory_start_addr
    }

    fn memory_size(&self) -> usize {
        self.object_start_addr + size_of::<CapabilityPool>() - self.memory_start_addr
    }
}

impl Drop for CapabilityPoolCapability {
    fn drop(&mut self) {
        unimplemented!();
    }
}

impl CapabilityPoolCapability {
    pub fn 
}
