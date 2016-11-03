use common::*;
use core::any::{Any, TypeId};
use core::ops::{Deref, DerefMut};
use util::{RwLock, align_up};
use util::managed_arc::{ManagedArc, ManagedArcAny, ManagedWeakPool256Arc};

use super::{UntypedDescriptor};

#[derive(Debug)]
pub struct CPoolDescriptor {
    weak_pool: ManagedWeakPool256Arc,
    next: Option<ManagedArcAny>,
}
pub type CPoolCap = ManagedArc<RwLock<CPoolDescriptor>>;

impl CPoolDescriptor {
    pub fn upgrade_any(&self, index: usize) -> Option<ManagedArcAny> {
        unsafe { self.weak_pool.read().upgrade_any(index, |ptr, type_id| { super::upgrade_any(ptr, type_id) }) }
    }

    pub fn upgrade<T: Any>(&self, index: usize) -> Option<ManagedArc<T>>
        where ManagedArc<T>: Any {
        self.weak_pool.read().upgrade(index)
    }

    pub fn downgrade_at<T: Any>(&self, arc: &ManagedArc<T>, index: usize)
        where ManagedArc<T>: Any {
        self.weak_pool.read().downgrade_at(arc, index)
    }

    pub fn downgrade_free<T: Any>(&self, arc: &ManagedArc<T>) -> Option<usize>
        where ManagedArc<T>: Any {
        self.weak_pool.read().downgrade_free(arc)
    }

    pub fn size(&self) -> usize {
        256
    }
}

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
