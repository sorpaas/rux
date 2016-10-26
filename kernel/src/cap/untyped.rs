use common::{PAddr, VAddr};
use util::{align_up};
use cap::{CapFull, MDB};

pub type UntypedFull = CapFull<UntypedHalf, [MDB; 1]>;

#[derive(Debug)]
pub struct UntypedHalf {
    start_paddr: PAddr,
    length: usize,
    watermark: PAddr,
}

impl UntypedFull {
    pub unsafe fn bootstrap(start_paddr: PAddr, length: usize) -> UntypedHalf {
        UntypedHalf {
            start_paddr: start_paddr,
            length: length,
            watermark: start_paddr,
        }
    }

    pub fn allocate(&mut self, length: usize, alignment: usize) -> (PAddr, Option<&mut MDB>) {
        let paddr = align_up(self.watermark, alignment);
        assert!(paddr + length <= self.start_paddr + self.length);

        self.watermark = paddr + length;
        (paddr, Some(self.mdb_mut(0)))
    }

    pub fn retype(untyped: &mut UntypedFull, length: usize, alignment: usize) -> (UntypedHalf, [Option<&mut MDB>; 1]) {
        let (start_paddr, mdb) = untyped.allocate(length, alignment);

        (UntypedHalf {
            start_paddr: start_paddr,
            length: length,
            watermark: start_paddr,
        }, [ mdb ])
    }
}

impl UntypedHalf {
    pub fn length(&self) -> usize {
        self.length
    }

    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }
}
