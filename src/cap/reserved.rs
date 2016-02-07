use common::*;

use super::{MemoryBlockPtr, MemoryBlockCapability};
use super::{PageFramePtr, PageFrameCapability};
use super::KernelReservedBlockCapability;
use super::KernelReservedFrameCapability;
use super::UntypedCapability;

use super::utils;

impl MemoryBlockPtr for KernelReservedFrameCapability {
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

impl MemoryBlockCapability for KernelReservedFrameCapability { }

impl MemoryBlockPtr for KernelReservedBlockCapability {
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

impl MemoryBlockCapability for KernelReservedBlockCapability { }

impl PageFramePtr for KernelReservedFrameCapability {
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

impl PageFrameCapability for KernelReservedFrameCapability { }

impl Drop for KernelReservedFrameCapability {
    fn drop(&mut self) {
        unimplemented!();
    }
}

impl KernelReservedFrameCapability {
    pub fn from_untyped(cap: UntypedCapability, frame_start_addr: PhysicalAddress, object_size: usize)
                        -> (Option<KernelReservedFrameCapability>, Option<UntypedCapability>) {
        assert!(frame_start_addr % PAGE_SIZE == 0);
        let frame_count = utils::necessary_page_count(object_size);
        let frame_size = frame_count * PAGE_SIZE;

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
                frame_count: frame_count }), ou2)
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
