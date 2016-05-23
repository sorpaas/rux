use common::*;
use core::ops::Drop;

use super::{Capability};
use super::utils;

pub struct MemoryBlock {
    physical_start_addr: PhysicalAddress,
    start_addr: PhysicalAddress,
    size: usize,
    useless: bool,
}

impl MemoryBlock {
    pub fn start_addr(&self) -> PhysicalAddress {
        self.start_addr
    }

    pub fn physical_start_addr(&self) -> PhysicalAddress {
        self.physical_start_addr
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn physical_size(&self) -> usize {
        self.size + (self.start_addr - self.physical_start_addr)
    }

    pub fn end_addr(&self) -> PhysicalAddress {
        self.start_addr + self.size - 1
    }

    pub unsafe fn mark_useless(&mut self) {
        self.useless = true;
    }

    const fn new(physical_start_addr: PhysicalAddress, start_addr: PhysicalAddress, size: usize) -> MemoryBlock {
        MemoryBlock { start_addr: start_addr, physical_start_addr: physical_start_addr, size: size,
                      useless: false }
    }

    pub const unsafe fn bootstrap(physical_start_addr: PhysicalAddress, size: usize) -> MemoryBlock {
        MemoryBlock::new(physical_start_addr, physical_start_addr, size)
    }
}

impl Drop for MemoryBlock {
    fn drop(&mut self) {
        if self.useless { return; }
    }
}

pub struct UntypedCapability {
    block: MemoryBlock,
    useless: bool
}

impl Capability for UntypedCapability { }

impl UntypedCapability {
    pub fn block(&self) -> &MemoryBlock {
        &self.block
    }

    pub const fn from_block(block: MemoryBlock) -> UntypedCapability {
        UntypedCapability { block: block, useless: false }
    }

    pub const unsafe fn bootstrap(physical_start_addr: PhysicalAddress, size: usize) -> UntypedCapability {
        UntypedCapability::from_block(MemoryBlock::bootstrap(physical_start_addr, size))
    }

    pub fn from_untyped(cap: &mut UntypedCapability, size: usize)
                        -> UntypedCapability {
        let block = cap.retype(1, size);
        UntypedCapability::from_block(block)
    }

    pub fn from_untyped_fixed(cap: &mut UntypedCapability, start_addr: PhysicalAddress, size: usize)
                              -> UntypedCapability {
        let block = cap.retype(start_addr, size);
        UntypedCapability::from_block(block)
    }

    pub unsafe fn mark_useless(&mut self) {
        self.useless = true;
    }

    pub fn retype(&mut self, alignment: usize, size: usize)
                  -> MemoryBlock {
        let target_physical_start_addr = self.block().start_addr();
        self.retype_fixed(utils::align(target_physical_start_addr, alignment), size)
    }

    pub fn retype_fixed(&mut self, start_addr: PhysicalAddress, size: usize)
                        -> MemoryBlock {
        let target_physical_start_addr = self.block().start_addr();
        let target_start_addr = start_addr;
        let target_size = size;
        let target = MemoryBlock::new(target_physical_start_addr, target_start_addr, target_size);
        assert!(target.end_addr() < self.block().end_addr());

        self.block.start_addr = target.end_addr() + 1;
        self.block.physical_start_addr = target.end_addr() + 1;
        self.block.size = self.block().end_addr() - start_addr + 1;

        target
    }

    pub fn merge(&mut self, mut cap: UntypedCapability) {
        assert!(self.block().end_addr() + 1 == cap.block().physical_start_addr());
        unsafe { cap.mark_useless(); }

        self.block.size += cap.block().physical_size();
    }
}

impl Drop for UntypedCapability {
    fn drop(&mut self) {
        unsafe { self.block.mark_useless(); }
        if self.useless { return; }

        unimplemented!();
    }
}
