use common::*;

use super::{MemoryBlockPtr, MemoryBlockCapability};
use super::KernelReservedBlockCapability;
use super::UntypedCapability;

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

impl Drop for KernelReservedBlockCapability {
    fn drop(&mut self) {
        unimplemented!();
    }
}

impl MemoryBlockCapability for KernelReservedBlockCapability { }

impl KernelReservedBlockCapability {
    pub fn from_untyped(cap: UntypedCapability, frame_start_addr: PhysicalAddress, frame_size: usize)
                        -> (Option<KernelReservedBlockCapability>, Option<UntypedCapability>) {
        assert!(frame_start_addr % PAGE_SIZE == 0);
        assert!(frame_size % PAGE_SIZE == 0);

        if frame_start_addr < cap.block_start_addr() || frame_start_addr + frame_size - 1 > cap.block_end_addr() {
            (None, Some(cap))
        } else {
            let block_start_addr = cap.block_start_addr();
            let block_size = frame_start_addr - block_start_addr + frame_size;
            let (mut u1, ou2) = UntypedCapability::from_untyped(cap, block_size);
            assert!(u1.block_size() == block_size);
            u1.block_size = 0;

            (Some(KernelReservedBlockCapability {
                block_start_addr: block_start_addr,
                block_size: block_size,
                frame_start_addr: frame_start_addr,
                frame_size: frame_size }), ou2)
        }
    }
}
