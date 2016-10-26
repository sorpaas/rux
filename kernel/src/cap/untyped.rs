use common::{PAddr, VAddr};
use util::{align_up};
use cap::{CapFull, MDB};

pub type UntypedFull<'a> = CapFull<UntypedHalf, [MDB<'a>; 1]>;

#[derive(Debug)]
pub struct UntypedHalf {
    start_paddr: PAddr,
    length: usize,
    watermark: PAddr,
}

impl<'a> UntypedFull<'a> {
    pub unsafe fn bootstrap(start_paddr: PAddr, length: usize) -> UntypedFull<'a> {
        Self::new(UntypedHalf {
            start_paddr: start_paddr,
            length: length,
            watermark: start_paddr,
        }, [ MDB::default() ])
    }

    pub fn allocate(&'a mut self, length: usize, alignment: usize) -> (PAddr, MDB<'a>) {
        let paddr = align_up(self.watermark, alignment);
        assert!(paddr + length <= self.start_paddr + self.length);

        self.watermark = paddr + length;
        (paddr, self.mdb_mut(0).derive())
    }

    pub fn retype(untyped: &'a mut UntypedFull<'a>, length: usize, alignment: usize) -> UntypedFull<'a> {
        let (start_paddr, mdb) = untyped.allocate(length, alignment);

        Self::new(UntypedHalf {
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
