use common::*;
use core::any::{Any, TypeId};
use core::ops::{Deref, DerefMut};
use util::{RwLock, align_up};
use util::managed_arc::{ManagedArc, ManagedArcAny, ManagedWeakPool256Arc};

use super::{UntypedDescriptor};

/// Capability pool descriptor.
#[derive(Debug)]
pub struct CPoolDescriptor {
    weak_pool: ManagedWeakPool256Arc,
    next: Option<ManagedArcAny>,
}
/// Capability pool capability. Reference-counted smart pointer to
/// capability pool descriptor. Capability pool itself is a
/// `ManagedWeakPool` with 256 entries.
///
/// Capability pool capability is used to hold multiple capabilities
/// together so as to be addressable in user-space programs.
pub type CPoolCap = ManagedArc<RwLock<CPoolDescriptor>>;

impl CPoolDescriptor {
    /// Create a new pointer to a capability descriptor using the
    /// index. If nothing is in the entry, `None` is returned.
    pub fn upgrade_any(&self, index: usize) -> Option<ManagedArcAny> {
        unsafe { self.weak_pool.read().upgrade_any(index, |ptr, type_id| { super::upgrade_any(ptr, type_id) }) }
    }

    /// Like `upgrade_any`, but returns a value with the specified
    /// type.
    pub fn upgrade<T: Any>(&self, index: usize) -> Option<ManagedArc<T>>
        where ManagedArc<T>: Any {
        self.weak_pool.read().upgrade(index)
    }

    /// Downgrade a capability into the capability pool (weak pool) at
    /// a specified index.
    pub fn downgrade_at<T: Any>(&self, arc: &ManagedArc<T>, index: usize)
        where ManagedArc<T>: Any {
        self.weak_pool.read().downgrade_at(arc, index)
    }

    /// Downgrade a capability into the capability pool (weak pool) at
    /// a free index.
    pub fn downgrade_free<T: Any>(&self, arc: &ManagedArc<T>) -> Option<usize>
        where ManagedArc<T>: Any {
        self.weak_pool.read().downgrade_free(arc)
    }

    /// Size of the capability pool.
    pub fn size(&self) -> usize {
        256
    }
}

impl CPoolCap {
    /// Create a capability pool capability from an untyped
    /// capability.
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
