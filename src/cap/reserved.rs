use common::*;

use super::{MemoryBlockCapability};
use super::{PageFrameCapability};
use super::KernelReservedBlockCapability;
use super::KernelReservedFrameCapability;
use super::UntypedCapability;

use super::paging::EntryFlags;
use super::paging::{Frame};
use super::utils;

impl MemoryBlockCapability for KernelReservedFrameCapability {
    fn block_start_addr(&self) -> PhysicalAddress {
        self.block_start_addr
    }

    fn block_size(&self) -> usize {
        self.block_size
    }
}

impl MemoryBlockCapability for KernelReservedBlockCapability {
    fn block_start_addr(&self) -> PhysicalAddress {
        self.block_start_addr
    }

    fn block_size(&self) -> usize {
        self.block_size
    }
}

struct KernelReservedFrameIterator {
    addr: PhysicalAddress,
    offset: usize,
    count: usize,
    flags: EntryFlags,
    guarded: Option<PhysicalAddress>,
}

impl Iterator for KernelReservedFrameIterator {
    type Item = Frame;
    fn next(&mut self) -> Option<Frame> {
        if self.count == 0 {
            None
        } else {
            let addr = self.addr;
            let offset = self.offset;

            self.offset = self.offset + 1;
            self.count = self.count - 1;
            self.addr = self.addr + PAGE_SIZE;

            match self.guarded {
                Some(x) => {
                    if x == addr {
                        self.next()
                    } else {
                        Some(Frame::new(addr, offset, self.flags))
                    }
                },
                None => {
                    Some(Frame::new(addr, offset, self.flags))
                }
            }
        }
    }
}

impl PageFrameCapability for KernelReservedFrameCapability {
    type FrameIterator = KernelReservedFrameIterator;
    fn frames(&self) -> KernelReservedFrameIterator {
        KernelReservedFrameIterator {
            addr: self.frame_start_addr,
            offset: 0,
            count: self.frame_count,
            flags: self.flags,
            guarded: self.guarded_frame_start_addr,
        }
    }
}

impl Drop for KernelReservedFrameCapability {
    fn drop(&mut self) {
        unimplemented!();
    }
}

impl Drop for KernelReservedBlockCapability {
    fn drop(&mut self) {
        unimplemented!();
    }
}

impl KernelReservedFrameCapability {
    pub fn from_untyped(cap: UntypedCapability, frame_start_addr: PhysicalAddress, object_size: usize,
                        guarded_frame_start_addr: Option<PhysicalAddress>, flags: EntryFlags)
                        -> (Option<KernelReservedFrameCapability>, Option<UntypedCapability>) {
        assert!(frame_start_addr % PAGE_SIZE == 0);
        let frame_count = utils::necessary_page_count(object_size);
        let frame_size = frame_count * PAGE_SIZE;

        match guarded_frame_start_addr {
            Some(x) => {
                assert!(x % PAGE_SIZE == 0);
                assert!(x >= frame_start_addr);
                assert!(x <= frame_start_addr + (frame_count - 1) * PAGE_SIZE);
            },
            None => { }
        }

        if frame_start_addr < cap.block_start_addr() || frame_start_addr + frame_size - 1 > cap.block_end_addr() {
            (None, Some(cap))
        } else {
            let block_start_addr = cap.block_start_addr();
            let block_size = frame_start_addr - block_start_addr + frame_size;
            let (mut u1, ou2) = UntypedCapability::from_untyped(cap, block_size);
            assert!(u1.block_size() == block_size);
            u1.block_size = 0;

            (Some(KernelReservedFrameCapability {
                block_start_addr: block_start_addr,
                block_size: block_size,
                frame_start_addr: frame_start_addr,
                frame_count: frame_count,
                guarded_frame_start_addr: guarded_frame_start_addr,
                flags: flags,
            }), ou2)
        }
    }
}

impl KernelReservedBlockCapability {
    pub fn from_untyped(cap: UntypedCapability, block_start_addr: PhysicalAddress, block_size: usize)
                        -> (Option<KernelReservedBlockCapability>, Option<UntypedCapability>) {
        if block_start_addr < cap.block_start_addr() || block_start_addr + block_size - 1 > cap.block_end_addr() {
            (None, Some(cap))
        } else {
            let real_block_start_addr = cap.block_start_addr();
            let real_block_size = block_start_addr - real_block_start_addr + block_size;
            let (mut u1, ou2) = UntypedCapability::from_untyped(cap, block_size);
            assert!(u1.block_size() == block_size);
            u1.block_size = 0;

            (Some(KernelReservedBlockCapability {
                block_start_addr: real_block_start_addr,
                block_size: real_block_size,
            }), ou2)
        }
    }
}
