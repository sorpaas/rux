use common::*;
use core::mem::{align_of, replace, uninitialized, size_of};
use core::ops::Drop;
use core::ptr;
use alloc::boxed::Box;

use super::MemoryBlockCapability;
use super::untyped::UntypedMemoryCapability;

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

pub struct CapabilityPool([Option<CapabilityUnion>; CAPABILITY_POOL_COUNT]);

// The main kernel capability pool is static. Other capability pools are created
// by retype kernel page.

pub struct CapabilityPoolCapability {
    start_addr: PhysicalAddress,
    physical_start_addr: PhysicalAddress,
    object: Box<CapabilityPool>,
}

impl MemoryBlockCapability for CapabilityPoolCapability {
    fn start_addr(&self) -> PhysicalAddress {
        self.start_addr
    }

    fn size(&self) -> usize {
        size_of::<CapabilityPool>()
    }

    fn physical_start_addr(&self) -> PhysicalAddress {
        self.physical_start_addr
    }

    fn physical_size(&self) -> usize {
        self.end_addr() - self.physical_start_addr()
    }
}

impl Drop for CapabilityPoolCapability {
    fn drop(&mut self) {
        unimplemented!();
    }
}

impl CapabilityPoolCapability {
    pub fn from_untyped(cap: UntypedMemoryCapability)
                        -> (Option<CapabilityPoolCapability>, Option<UntypedMemoryCapability>) {
        let size = size_of::<CapabilityPool>();
        let align = align_of::<CapabilityPool>();
        let start_addr = cap.start_addr() + (align - cap.start_addr() % align);
        let end_addr = start_addr + size;

        if end_addr > cap.end_addr() {
            (None, Some(cap))
        } else {
            let pool_box = unsafe {
                let mut pool_array: [Option<CapabilityUnion>; CAPABILITY_POOL_COUNT] = uninitialized();

                for (i, element) in pool_array.iter_mut().enumerate() {
                    let cap: Option<CapabilityUnion> = None;

                    ptr::write(element, cap)
                }

                replace::<CapabilityPool>(&mut *(start_addr as *mut _), CapabilityPool(pool_array));
                Box::from_raw(*(start_addr as *mut _))
            };

            let pool_cap = CapabilityPoolCapability {
                start_addr: start_addr,
                physical_start_addr: cap.start_addr(),
                object: pool_box,
            };

            let cap = UntypedMemoryCapability::resize(cap, &pool_cap);
            (Some(pool_cap), cap)
        }
    }
}
