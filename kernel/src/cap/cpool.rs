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

fn downgrade_at_owning<T: Any>(arc: ManagedArc<T>, index: usize, desc: &CPoolDescriptor)
    where ManagedArc<T>: Any {
    desc.downgrade_at(&arc, index)
}

fn downgrade_free_owning<T: Any>(arc: ManagedArc<T>, desc: &CPoolDescriptor) -> Option<usize>
    where ManagedArc<T>: Any {
    desc.downgrade_free(&arc)
}

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

    /// Downgrade a `ManagedArcAny` into the capability pool (weak
    /// pool) at a specified index.
    pub fn downgrade_any_at(&self, arc: ManagedArcAny, index: usize) {
        doto_any!(arc, downgrade_at_owning, index, self)
    }

    /// Downgrade a `ManagedArcAny` into the capability pool (weak
    /// pool) at a free index.
    pub fn downgrade_any_free(&self, arc: ManagedArcAny) -> Option<usize> {
        doto_any!(arc, downgrade_free_owning, self)
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

    fn lookup<R, F: FnOnce(Option<(&CPoolDescriptor, usize)>) -> R>(&self, caddr: CAddr, f: F) -> R {
        if caddr.1 == 0 {
            f(None)
        } else if caddr.1 == 1 {
            let cur_lookup_index = caddr.0[0];
            f(Some((self.read().deref(), cur_lookup_index as usize)))
        } else {
            let cur_lookup_index = caddr.0[0];
            let next_lookup_cpool: Option<CPoolCap> = self.read().upgrade(cur_lookup_index as usize);
            let next_caddr = caddr << 1;

            if next_lookup_cpool.is_some() {
                let next_lookup_cpool = next_lookup_cpool.unwrap();
                next_lookup_cpool.lookup::<R, F>(next_caddr, f)
            } else {
                f(None)
            }
        }
    }

    /// Lookup upgrading a capability from a capability address to a `ManagedArcAny`.
    pub fn lookup_upgrade_any(&self, caddr: CAddr) -> Option<ManagedArcAny> {
        self.lookup(caddr, |data| {
            data.map_or(None, |(cpool, index)| {
                cpool.upgrade_any(index)
            })
        })
    }

    /// Lookup upgrading a capability from a capability address.
    pub fn lookup_upgrade<T: Any>(&self, caddr: CAddr) -> Option<ManagedArc<T>> {
        self.lookup(caddr, |data| {
            data.map_or(None, |(cpool, index)| {
                cpool.upgrade(index)
            })
        })
    }

    /// Downgrade a capability into the capability pool at a specified capability address.
    pub fn lookup_downgrade_at<T: Any>(&self, arc: &ManagedArc<T>, caddr: CAddr)
        where ManagedArc<T>: Any {
        self.lookup(caddr, |data| {
            let (cpool, index) = data.unwrap();
            cpool.downgrade_at(arc, index);
        });
    }
}
