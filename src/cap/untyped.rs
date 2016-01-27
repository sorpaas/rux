use common::*;
use core::ops::Drop;

use super::CapabilityUnion;
use super::{AllocatableCapability, MemoryBlockCapability};

/// Untyped memory and page table are memory management tricks, those are not
/// actually accessible in the virtual memory.

pub struct UntypedMemoryCapability {
    start_addr: PhysicalAddress,
    size: usize,
}

impl MemoryBlockCapability for UntypedMemoryCapability {
    fn start_addr(&self) -> PhysicalAddress {
        self.start_addr
    }

    fn size(&self) -> usize {
        self.size
    }

    fn physical_start_addr(&self) -> PhysicalAddress {
        self.start_addr
    }

    fn physical_size(&self) -> usize {
        self.size
    }
}

impl Drop for UntypedMemoryCapability {
    fn drop(&mut self) {
        unimplemented!();
    }
}

impl UntypedMemoryCapability {
    pub fn from_untyped(cap: UntypedMemoryCapability, size: usize)
                        -> (UntypedMemoryCapability, Some<UntypedMemoryCapability>) {
        if cap.start_addr() + size + 1 >= cap.end_addr() {
            (cap, None)
        } else {
            let new_cap = UntypedMemoryCapability {
                start_addr: cap.start_addr(),
                size: size,
            };
            cap.start_addr = cap.start_addr() + size + 1;
            cap.size = cap.size() - size;

            (new_cap, Some(cap))
        }
    }
}
