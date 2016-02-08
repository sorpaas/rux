use common::*;
use core::marker::PhantomData;
use core::mem::{align_of, replace, uninitialized, size_of};

use super::{MemoryBlockCapability};
use super::{PageFrameCapability};
use super::{PageObjectCapability};
use super::{UntypedCapability};
use super::utils;
use super::paging::EntryFlags;
use super::paging::{Frame};

impl<T> MemoryBlockCapability for PageObjectCapability<T> {
    fn block_start_addr(&self) -> PhysicalAddress {
        self.block_start_addr
    }

    fn block_size(&self) -> usize {
        self.block_size
    }
}

struct ContinuousFrameIterator {
    addr: PhysicalAddress,
    offset: usize,
    count: usize,
    flags: EntryFlags,
}

impl Iterator for ContinuousFrameIterator {
    type Item = Frame;
    fn next(&mut self) -> Option<Frame> {
        if self.count == 0 {
            None
        } else {
            let addr = self.addr;
            let offset = self.offset;

            self.addr = self.addr + PAGE_SIZE;
            self.offset = self.offset + 1;
            self.count = self.count - 1;

            Some(Frame::new(addr, offset, self.flags))
        }
    }
}

impl<T> PageFrameCapability for PageObjectCapability<T> {
    type FrameIterator = ContinuousFrameIterator;
    fn frames(&self) -> ContinuousFrameIterator {
        ContinuousFrameIterator {
            addr: self.frame_start_addr,
            offset: 0,
            count: self.frame_count,
            flags: self.flags,
        }
    }
}

impl<T> Drop for PageObjectCapability<T> {
    fn drop(&mut self) {
        unimplemented!();
    }
}

impl<T> PageObjectCapability<T> {
    pub fn object_size() -> usize {
        size_of::<T>()
    }

    pub fn from_untyped_switching(untyped: UntypedCapability, flags: EntryFlags) -> PageObjectCapability<T> {
        let page_start_addr = utils::necessary_page_start_addr(untyped.block_start_addr());
        let page_count = utils::necessary_page_count(Self::object_size());
        let block_size = utils::necessary_block_size(untyped.block_start_addr(), page_count);
        let page_size = page_count * PAGE_SIZE;

        assert!(untyped.block_size() == block_size);

        let pool_cap = PageObjectCapability::<T> {
            block_start_addr: untyped.block_start_addr(),
            block_size: block_size,
            frame_start_addr: page_start_addr,
            frame_count: page_count,
            flags: flags,
            _marker: PhantomData::<T>
        };

        let mut untyped = untyped;
        untyped.block_size = 0;

        pool_cap
    }

    pub fn from_untyped(untyped: UntypedCapability, flags: EntryFlags)
                        -> (Option<PageObjectCapability<T>>, Option<UntypedCapability>) {
        let page_count = utils::necessary_page_count(Self::object_size());
        let block_size = utils::necessary_block_size(untyped.block_start_addr(), page_count);

        let (u1, u2) = UntypedCapability::from_untyped(untyped, block_size);

        if u1.block_size() < block_size {
            assert!(u2.is_none());
            (None, Some(u1))
        } else {
            (Some(Self::from_untyped_switching(u1, flags)), u2)
        }
    }
}
