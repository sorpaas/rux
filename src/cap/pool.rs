use common::*;
use core::mem::{align_of, replace, uninitialized, size_of};

use super::{MemoryBlockPtr, MemoryBlockCapability};
use super::{PageBlockPtr, PageBlockCapability};
use super::{UntypedCapability};
use super::{CapabilityPoolCapability, CapabilityUnion};
use super::utils;

pub struct CapabilityPool {
    capabilities: [Option<CapabilityUnion>; CAPABILITY_POOL_COUNT],
    referred: bool,
}

impl MemoryBlockPtr for CapabilityPoolCapability {
    fn get_block_start_addr(&self) -> PhysicalAddress {
        self.block_start_addr
    }

    fn set_block_start_addr(&mut self, addr: PhysicalAddress) {
        self.block_start_addr = addr
    }

    fn get_block_size(&self) -> usize {
        self.block_size
    }

    fn set_block_size(&mut self, size: usize) {
        self.block_size = size
    }
}

impl MemoryBlockCapability for CapabilityPoolCapability { }

impl PageBlockPtr for CapabilityPoolCapability {
    fn get_page_start_addr(&self) -> PhysicalAddress {
        self.page_start_addr
    }

    fn set_page_start_addr(&mut self, addr: PhysicalAddress) {
        self.page_start_addr = addr;
    }

    fn get_page_counts(&self) -> usize {
        self.page_counts
    }

    fn set_page_counts(&mut self, counts: usize) {
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
    pub fn from_untyped_switching(untyped: UntypedCapability) -> CapabilityPoolCapability {
        let page_start_addr = utils::necessary_page_start_addr(untyped.block_start_addr());
        let page_counts = utils::necessary_page_counts(Self::object_size());
        let block_size = utils::necessary_block_size(untyped.block_start_addr(), page_counts);
        let page_size = page_counts * PAGE_SIZE;

        assert!(untyped.block_size() == block_size);

        let pool_cap = CapabilityPoolCapability {
            block_start_addr: untyped.block_start_addr(),
            block_size: block_size,
            page_start_addr: page_start_addr,
            page_counts: page_counts,
        };

        let mut untyped = untyped;
        untyped.block_size = 0;

        pool_cap
    }

    pub fn from_untyped(untyped: UntypedCapability)
                        -> (Option<CapabilityPoolCapability>, Option<UntypedCapability>) {
        let page_counts = utils::necessary_page_counts(Self::object_size());
        let block_size = utils::necessary_block_size(untyped.block_start_addr(), page_counts);

        let (u1, u2) = UntypedCapability::from_untyped(untyped, block_size);

        if u1.block_size() < block_size {
            assert!(u2.is_none());
            (None, Some(u1))
        } else {
            (Some(Self::from_untyped_switching(u1)), u2)
        }
    }
}
