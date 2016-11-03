mod untyped;
mod cpool;

pub use self::untyped::{UntypedDescriptor, UntypedCap};
pub use self::cpool::{CPoolDescriptor, CPoolCap};

use util::managed_arc::{ManagedWeakPool256Arc, ManagedArcAny};

pub fn upgrade_any(weak_pool: &ManagedWeakPool256Arc, index: usize) -> Option<ManagedArcAny> {
    if let Some(r) = weak_pool.upgrade::<CPoolCap>(index) {
        Some(r.into())
    } else if let Some(r) = weak_pool.upgrade::<UntypedCap>(index) {
        Some(r.into())
    } else {
        None
    }
}
