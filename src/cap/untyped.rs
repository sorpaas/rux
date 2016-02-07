use common::*;
use core::ops::Drop;

use super::{MemoryBlockPtr, MemoryBlockCapability};
use super::UntypedCapability;

impl MemoryBlockPtr for UntypedCapability {
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

impl MemoryBlockCapability for UntypedCapability { }

impl Drop for UntypedCapability {
    fn drop(&mut self) {
        if self.block_size() == 0 { return }

        unimplemented!();
    }
}

impl UntypedCapability {
    pub unsafe fn new(block_start_addr: usize, block_size: usize) -> UntypedCapability {
        UntypedCapability { block_start_addr: block_start_addr, block_size: block_size }
    }

    pub fn from_untyped_three(cap: UntypedCapability, block_start_addr: usize, block_size: usize)
                              -> (UntypedCapability, Option<UntypedCapability>, Option<UntypedCapability>) {
        assert!(block_start_addr >= cap.block_start_addr(),
                "Requested block start address must be after the original capability.");
        assert!(block_start_addr + block_size <= cap.block_end_addr(),
                "Requested block end address must be before the original capability.");
        assert!(block_size > 0,
                "Block size must be greater than 0.");

        let u1_start_addr = cap.block_start_addr();
        let u1_size = block_start_addr - cap.block_start_addr();
        let u2_start_addr = block_start_addr;
        let u2_size = block_size;
        let u3_start_addr = u2_start_addr + u2_size;
        let u3_size = cap.block_end_addr() - u3_start_addr + 1;

        let mut cap = cap;
        cap.block_start_addr = u2_start_addr;
        cap.block_size = u2_size;

        if u1_size > 0 && u3_size > 0 {
            (cap,
             Some(UntypedCapability { block_start_addr: u1_start_addr, block_size: u1_size }),
             Some(UntypedCapability { block_start_addr: u3_start_addr, block_size: u3_size }))
        } else if u1_size > 0 {
            (cap,
             Some(UntypedCapability { block_start_addr: u1_start_addr, block_size: u1_size }),
             None)
        } else if u3_size > 0 {
            (cap,
             Some(UntypedCapability { block_start_addr: u3_start_addr, block_size: u3_size}),
             None)
        } else {
            (cap,
             None,
             None)
        }
    }

    pub fn from_untyped(cap: UntypedCapability, block_size: usize)
                        -> (UntypedCapability, Option<UntypedCapability>) {
        let block_start_addr = cap.block_start_addr();
        let tuple = UntypedCapability::from_untyped_three(cap, block_start_addr, block_size);
        assert!(tuple.2.is_none(), "According to logic, the third item of the tuple should be none.");

        (tuple.0, tuple.1)
    }
}
