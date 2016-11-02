use common::*;
use util::{align_up};
use util::managed_arc::{ManagedArc, ManagedArcAny};
use spin::{RwLock};

#[derive(Debug)]
pub struct UntypedDescriptor {
    start_paddr: PAddr,
    length: usize,
    watermark: PAddr,
    first_child: Option<ManagedArcAny>
}
pub type UntypedCap = ManagedArc<RwLock<UntypedDescriptor>>;

impl UntypedCap {
    pub unsafe fn bootstrap(start_paddr: PAddr, length: usize) -> Self {
        let des_paddr = align_up(start_paddr, UntypedCap::inner_alignment());
        assert!(des_paddr + UntypedCap::inner_length() <= start_paddr + length);

        log!("des_paddr: {:?}", des_paddr);

        Self::new(des_paddr, RwLock::new(UntypedDescriptor {
            start_paddr: start_paddr,
            length: length,
            watermark: des_paddr + UntypedCap::inner_length(),
            first_child: None,
        }))
    }
}

impl UntypedDescriptor {
    pub unsafe fn allocate(&mut self, length: usize, alignment: usize) -> PAddr {
        let paddr = align_up(self.watermark, alignment);
        assert!(paddr + length <= self.start_paddr + self.length);

        self.watermark = paddr + length;
        paddr
    }

    pub unsafe fn derive<F>(&mut self, length: usize, alignment: usize, f: F) where F: FnOnce(PAddr, Option<ManagedArcAny>) -> ManagedArcAny {
        let paddr = self.allocate(length, alignment);
        self.first_child = Some(f(paddr, self.first_child.take()));
    }
}
