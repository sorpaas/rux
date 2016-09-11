use common::{PAddr, VAddr};
use utils::{align_up};

#[derive(Debug)]
pub struct UntypedHalf {
    start_paddr: PAddr,
    length: usize,
    watermark: PAddr,
}

impl UntypedHalf {
    pub fn new(start_paddr: PAddr, length: usize) -> UntypedHalf {
        UntypedHalf {
            start_paddr: start_paddr,
            length: length,
            watermark: start_paddr
        }
    }

    pub fn allocate(&mut self, length: usize, alignment: usize) -> PAddr {
        let paddr = align_up(self.watermark, alignment);
        assert!(paddr + length <= self.start_paddr + self.length);
        
        self.watermark = paddr + length;
        paddr
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }
}
