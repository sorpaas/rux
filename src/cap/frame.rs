use common::*;

use super::{MemoryBlock, UntypedCapability,
            Capability};

pub struct FrameCapability {
    block: MemoryBlock,
    flags: EntryFlags,
}

impl Capability for FrameCapability { }

impl FrameCapability {
    pub const fn from_block(block: MemoryBlock, flags: EntryFlags) -> FrameCapability {
        FrameCapability { block: block, flags: flags }
    }

    pub fn from_untyped(cap: &mut UntypedCapability, count: usize, flags: EntryFlags)
                        -> FrameCapability {
        let block = cap.retype(PAGE_SIZE, count * PAGE_SIZE);
        FrameCapability::from_block(block, flags)
    }

    pub fn from_untyped_fixed(cap: &mut UntypedCapability, start_addr: PhysicalAddress, count: usize, flags: EntryFlags)
                              -> FrameCapability {
        assert!(start_addr % PAGE_SIZE == 0);
        let block = cap.retype_fixed(start_addr, count * PAGE_SIZE);
        FrameCapability::from_block(block, flags)
    }

    pub fn block(&self) -> &MemoryBlock {
        &self.block
    }

    pub fn count(&self) -> usize {
        self.block().size() / PAGE_SIZE
    }

    pub fn flags(&self) -> EntryFlags {
        self.flags
    }
}

pub struct GuardedCapability {
    block: MemoryBlock,
}

impl Capability for GuardedCapability { }

impl GuardedCapability {
    pub const fn from_block(block: MemoryBlock) -> GuardedCapability {
        GuardedCapability { block: block }
    }

    pub fn from_untyped(cap: &mut UntypedCapability, size: usize)
                        -> GuardedCapability {
        let block = cap.retype(1, size);
        GuardedCapability::from_block(block)
    }

    pub fn from_untyped_fixed(cap: &mut UntypedCapability, start_addr: PhysicalAddress, size: usize)
                              -> GuardedCapability {
        let block = cap.retype(start_addr, size);
        GuardedCapability::from_block(block)
    }

    pub fn block(&self) -> &MemoryBlock {
        &self.block
    }

    pub fn count(&self) -> usize {
        self.block().size() / PAGE_SIZE
    }
}
