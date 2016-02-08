use common::*;
use super::paging::{Frame, EntryFlags};

pub fn necessary_page_count(object_size: usize) -> usize {
    if object_size % PAGE_SIZE == 0 {
        object_size / PAGE_SIZE
    } else {
        object_size / PAGE_SIZE + 1
    }
}

pub fn necessary_block_size(addr: PhysicalAddress, page_counts: usize) -> usize {
    let page_start_addr = necessary_page_start_addr(addr);
    let page_size = page_counts * PAGE_SIZE;

    (page_start_addr - addr) + page_size
}

pub fn necessary_page_start_addr(addr: PhysicalAddress) -> PhysicalAddress {
    if addr % PAGE_SIZE == 0 {
        addr
    } else {
        addr + (PAGE_SIZE - addr % PAGE_SIZE)
    }
}

pub struct ContinuousFrameIterator {
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

impl ContinuousFrameIterator {
    pub fn new(addr: PhysicalAddress, count: usize, flags: EntryFlags) -> ContinuousFrameIterator {
        ContinuousFrameIterator {
            addr: addr,
            offset: 0,
            count: count,
            flags: flags,
        }
    }

}
