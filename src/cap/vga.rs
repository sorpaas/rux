use common::*;

use super::VGABufferCapability;
use super::{MemoryBlockCapability, PageFrameCapability};
use super::utils;
use super::utils::ContinuousFrameIterator;

impl MemoryBlockCapability for VGABufferCapability {
    fn block_start_addr(&self) -> PhysicalAddress {
        0xb8000
    }

    fn block_size(&self) -> usize {
        PAGE_SIZE
    }
}

impl PageFrameCapability for VGABufferCapability {
    type FrameIterator = ContinuousFrameIterator;
    fn frames(&self) -> ContinuousFrameIterator {
        use super::paging::entry::WRITABLE;
        ContinuousFrameIterator::new(0xb8000, 1, WRITABLE)
    }
}

impl VGABufferCapability {
    pub unsafe fn new() -> VGABufferCapability {
        VGABufferCapability
    }
}
