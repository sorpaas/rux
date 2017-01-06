use common::*;
use util::{RwLock, align_up};
use util::managed_arc::{ManagedArc, ManagedArcAny};

/// Untyped descriptor.
#[derive(Debug)]
pub struct UntypedDescriptor {
    start_paddr: PAddr,
    length: usize,
    watermark: PAddr,
    first_child: Option<ManagedArcAny>
}
/// Untyped capability. Reference-counted smart pointer to untyped
/// descriptor.
///
/// Untyped capability represents free memory that can be retyped to
/// different useful capabilities.
pub type UntypedCap = ManagedArc<RwLock<UntypedDescriptor>>;

impl UntypedCap {
    /// Bootstrap an untyped capability using a memory region information.
    ///
    /// # Safety
    ///
    /// Can only be used for free memory regions returned from
    /// `InitInfo`.
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
    /// Length of the untyped region.
    pub fn length(&self) -> usize {
        self.length
    }

    /// Start physical address of the untyped region.
    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }

    /// Allocate a memory region using the given length and
    /// alignment. Shift the watermark of the current descriptor
    /// passing over the allocated region.
    pub unsafe fn allocate(&mut self, length: usize, alignment: usize) -> PAddr {
        let paddr = align_up(self.watermark, alignment);
        assert!(paddr + length <= self.start_paddr + self.length);

        self.watermark = paddr + length;
        paddr
    }

    /// Derive and allocate a memory region to a capability that
    /// requires memory region.
    pub unsafe fn derive<F>(&mut self, length: usize, alignment: usize, f: F) where F: FnOnce(PAddr, Option<ManagedArcAny>) -> ManagedArcAny {
        let paddr = self.allocate(length, alignment);
        self.first_child = Some(f(paddr, self.first_child.take()));
    }
}
