use common::*;
use core::ops::Drop;

use super::{MemoryBlockCapability};

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
