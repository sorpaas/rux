use common::*;
use core::marker::PhantomData;
use core::mem::{align_of, replace, uninitialized, size_of};

use super::{MemoryBlockPtr, MemoryBlockCapability};
use super::{PageFramePtr, PageFrameCapability};
use super::{PageObjectCapability};
use super::{UntypedCapability};
use super::utils;

impl<T> MemoryBlockPtr for PageObjectCapability<T> {
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

impl<T> MemoryBlockCapability for PageObjectCapability<T> { }

impl<T> PageFramePtr for PageObjectCapability<T> {
    fn get_frame_start_addr(&self) -> PhysicalAddress {
        self.frame_start_addr
    }

    fn set_frame_start_addr(&mut self, addr: PhysicalAddress) {
        self.frame_start_addr = addr;
    }

    fn get_frame_count(&self) -> usize {
        self.frame_count
    }

    fn set_frame_count(&mut self, count: usize) {
        self.frame_count = count
    }
}

impl<T> PageFrameCapability for PageObjectCapability<T> { }

impl<T> Drop for PageObjectCapability<T> {
    fn drop(&mut self) {
        unimplemented!();
    }
}

impl<T> PageObjectCapability<T> {
    pub fn object_size() -> usize {
        size_of::<T>()
    }

    pub fn from_untyped_switching(untyped: UntypedCapability) -> PageObjectCapability<T> {
        let page_start_addr = utils::necessary_page_start_addr(untyped.block_start_addr());
        let page_counts = utils::necessary_page_counts(Self::object_size());
        let block_size = utils::necessary_block_size(untyped.block_start_addr(), page_counts);
        let page_size = page_counts * PAGE_SIZE;

        assert!(untyped.block_size() == block_size);

        let pool_cap = PageObjectCapability::<T> {
            block_start_addr: untyped.block_start_addr(),
            block_size: block_size,
            frame_start_addr: page_start_addr,
            frame_count: page_counts,
            _marker: PhantomData::<T>
        };

        let mut untyped = untyped;
        untyped.block_size = 0;

        pool_cap
    }

    pub fn from_untyped(untyped: UntypedCapability)
                        -> (Option<PageObjectCapability<T>>, Option<UntypedCapability>) {
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
