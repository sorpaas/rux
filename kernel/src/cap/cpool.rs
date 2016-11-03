use common::*;
use core::any::{Any, TypeId};
use util::{RwLock, align_up};
use util::managed_arc::{ManagedArc, ManagedArcAny, ManagedWeakPool256Arc};

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

    pub fn size(&self) -> usize {
        256
    }

    pub fn upgrade_any(&self, index: usize) -> Option<ManagedArcAny> {
        let cpool = self.read();

        super::upgrade_any(&cpool.weak_pool, index)
    }

    pub fn upgrade<T: Any>(&self, index: usize) -> Option<ManagedArc<T>>
        where ManagedArc<T>: Any {
        self.read().weak_pool.upgrade(index)
    }

    pub fn downgrade_at<T: Any>(&self, arc: &ManagedArc<T>, index: usize)
        where ManagedArc<T>: Any {
        self.read().weak_pool.downgrade_at(arc, index)
    }

    pub fn downgrade_free<T: Any>(&self, arc: &ManagedArc<T>) -> Option<usize>
        where ManagedArc<T>: Any {
        self.read().weak_pool.downgrade_free(arc)
    }
}
