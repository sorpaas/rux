use common::*;
use core::mem::{align_of, replace, uninitialized, size_of};
use core::ops::Drop;
use core::ptr;
use alloc::boxed::Box;

use super::MemoryBlockCapability;
use super::untyped::UntypedMemoryCapability;

pub struct CapabilityPool {
    capabilities: [Option<CapabilityUnion>; CAPABILITY_POOL_COUNT],
    referred: bool,
}

impl MemoryBlockPtr for CapabilityPoolCapability {
    fn get_block_start_addr(&self) -> PhysicalAddress {
        self.block_start_addr
    }

    fn set_block_start_addr(&self, addr: PhysicalAddress) {
        self.block_start_addr = addr
    }

    fn get_block_size(&self) -> usize {
        self.block_size
    }

    fn set_block_size(&self, size: usize) {
        self.block_size = size
    }
}

impl MemoryBlockCapability for CapabilityPoolCapability { }

impl PageBlockPtr for CapabilityPoolCapability {
    fn get_page_start_addr(&self) -> PhysicalAddress {
        self.page_start_addr
    }

    fn set_page_start_addr(&self, addr: PhysicalAddress) {
        self.page_start_addr = addr;
    }

    fn get_page_counts(&self) -> usize {
        self.page_counts
    }

    fn set_page_counts(&self, counts: usize) {
        self.page_counts = counts;
    }
}

impl PageBlockCapability<[Option<CapabilityUnion>; CAPABILITY_POOL_COUNT]> for CapabilityPoolCapability { }

impl Drop for CapabilityPoolCapability {
    fn drop(&mut self) {
        unimplemented!();
    }
}

impl CapabilityPoolCapability {
    pub fn from_untyped_switching(untyped: UntypedMemoryCapability) -> CapabilityPoolCapability {
        let page_start_addr = Self.necessary_page_start_addr(untyped.block_start_addr());
        let page_counts = Self.necessary_page_counts();
        let block_size = Self.necessary_block_size();
        let page_size = page_counts * PAGE_SIZE;

        assert!("The untyped capability must fit exactly.", untyped.block_size() == block_size);

        let pool_cap = CapabilityPoolCapability {
            block_start_addr: untyped.block_start_addr(),
            block_size: block_size,
            page_start_addr: page_start_addr,
            page_counts: page_counts,
        };
        untyped.block_size = 0;

        pool_cap
    }

    pub fn from_untyped(untyped: UntypedMemoryCapability)
                        -> (Option<CapabilityPoolCapability>, Option<UntypedMemoryCapability>) {
        let page_counts = Self.necessary_page_counts();
        let block_size = Self.necessary_block_size();

        let (u1, u2) = UntypedMemoryCapability.from_untyped(untyped, block_size);

        if u1.block_size() < block_size {
            assert!("According to logic, u2 should be none.", u2.is_none());
            (None, Some(u1))
        } else {
            (Self.from_untyped_switching(u1), u2)
        }
    }
}
