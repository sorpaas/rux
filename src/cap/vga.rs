use common::*;

use super::{MemoryBlock};
use super::utils;

pub struct VGABufferCapability {
    block: MemoryBlock,
}

impl VGABufferCapability {
    pub const unsafe fn bootstrap() -> VGABufferCapability {
        VGABufferCapability { block: MemoryBlock::bootstrap(0xb8000, PAGE_SIZE) }
    }
}
