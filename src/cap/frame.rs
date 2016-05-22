use common::*;

use super::{MemoryBlock, UntypedCapability,
            Capability};

pub struct FrameCapability {
    block: MemoryBlock,
    count: usize,
    flags: EntryFlags,
}

impl Capability for FrameCapability { }

impl FrameCapability {
    pub fn from_untyped(cap: UntypedCapability, count: usize, flags: EntryFlags)
                        -> (FrameCapability, Option<UntypedCapability>) {
        let (block, remained) = UntypedCapability::retype(cap, PAGE_SIZE, count * PAGE_SIZE);
        (FrameCapability { block: block, count: count, flags: flags }, remained)
    }

    pub fn from_untyped_fixed(cap: UntypedCapability, start_addr: PhysicalAddress, count: usize, flags: EntryFlags)
                              -> (FrameCapability, Option<UntypedCapability>) {
        assert!(start_addr % PAGE_SIZE == 0);
        let (block, remained) = UntypedCapability::retype_fixed(cap, start_addr, count * PAGE_SIZE);
        (FrameCapability { block: block, count: count, flags: flags }, remained)
    }

    pub fn block(&self) -> &MemoryBlock {
        &self.block
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn flags(&self) -> EntryFlags {
        self.flags
    }
}

pub struct GuardedFrameCapability {
    block: MemoryBlock,
}

impl Capability for GuardedFrameCapability { }

impl GuardedFrameCapability {
    pub fn from_untyped(cap: UntypedCapability, size: usize)
                        -> (GuardedFrameCapability, Option<UntypedCapability>) {
        let (block, remained) = UntypedCapability::retype(cap, 1, size);
        (GuardedFrameCapability { block: block }, remained)
    }

    pub fn from_untyped_fixed(cap: UntypedCapability, start_addr: PhysicalAddress, size: usize)
                              -> (GuardedFrameCapability, Option<UntypedCapability>) {
        let (block, remained) = UntypedCapability::retype(cap, start_addr, size);
        (GuardedFrameCapability { block: block }, remained)
    }

    pub fn block(&self) -> &MemoryBlock {
        &self.block
    }
}
