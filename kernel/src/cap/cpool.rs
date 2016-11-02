use common::*;
use util::{align_up};
use util::managed_arc::{ManagedArc, ManagedArcAny, ManagedWeakPool256Arc};
use spin::{RwLock};

use super::{UntypedDescriptor};

#[derive(Debug)]
pub struct CPoolDescriptor {
    weak_pool: ManagedWeakPool256Arc,
    next: Option<ManagedArcAny>,
}
pub type CPoolCap = ManagedArc<RwLock<CPoolDescriptor>>;

impl CPoolCap {
    pub fn retype_from(untyped: &mut UntypedDescriptor) -> Self {
        let mut arc: Option<Self> = None;

        let weak_pool = unsafe { ManagedWeakPool256Arc::create(
            untyped.allocate(ManagedWeakPool256Arc::inner_length(),
                             ManagedWeakPool256Arc::inner_alignment())) };

        unsafe { untyped.derive(Self::inner_length(), Self::inner_alignment(), |paddr, next_child| {
            arc = Some(unsafe {
                Self::new(paddr, RwLock::new(CPoolDescriptor {
                    weak_pool: weak_pool,
                    next: next_child,
                }))
            });

            arc.clone().unwrap().into()
        }) };

        arc.unwrap()
    }
}
