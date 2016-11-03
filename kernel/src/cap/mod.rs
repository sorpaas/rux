mod untyped;
mod cpool;

pub use self::untyped::{UntypedDescriptor, UntypedCap};
pub use self::cpool::{CPoolDescriptor, CPoolCap};
pub use arch::cap::{TopPageTableCap, PageCap};

use arch;
use util::managed_arc::{ManagedWeakPool256Arc, ManagedArcAny, ManagedArc};

pub fn upgrade_any(weak_pool: &ManagedWeakPool256Arc, index: usize) -> Option<ManagedArcAny> {
    if let Some(r) = weak_pool.upgrade(index): Option<CPoolCap> {
        Some(r.into())
    } else if let Some(r) = weak_pool.upgrade(index): Option<UntypedCap> {
        Some(r.into())
    } else {
        arch::cap::upgrade_any(weak_pool, index)
    }
}

pub fn drop_any(any: ManagedArcAny) {
    if any.is::<CPoolCap>() {
        any.into(): CPoolCap;
    } else if any.is::<UntypedCap>() {
        any.into(): UntypedCap;
    } else {
        arch::cap::drop_any(any);
    }
}
