mod paging;

pub use self::paging::{PML4Descriptor, PML4Cap,
                       PDPTDescriptor, PDPTCap,
                       PDDescriptor, PDCap,
                       PTDescriptor, PTCap,
                       PageDescriptor, PageCap};

pub type TopPageTableCap = PML4Cap;

use util::managed_arc::{ManagedWeakPool256Arc, ManagedArcAny};

pub fn upgrade_any(weak_pool: &ManagedWeakPool256Arc, index: usize) -> Option<ManagedArcAny> {
    if let Some(r) = weak_pool.upgrade(index): Option<PML4Cap> {
        Some(r.into())
    } else if let Some(r) = weak_pool.upgrade(index): Option<PDPTCap> {
        Some(r.into())
    } else if let Some(r) = weak_pool.upgrade(index): Option<PDCap> {
        Some(r.into())
    } else if let Some(r) = weak_pool.upgrade(index): Option<PTCap> {
        Some(r.into())
    } else if let Some(r) = weak_pool.upgrade(index): Option<PageCap> {
        Some(r.into())
    } else {
        None
    }
}

pub fn drop_any(any: ManagedArcAny) {
    if any.is::<PML4Cap>() {
        any.into(): PML4Cap;
    } else if any.is::<PDPTCap>() {
        any.into(): PDPTCap;
    } else if any.is::<PDCap>() {
        any.into(): PDCap;
    } else if any.is::<PTCap>() {
        any.into(): PTCap;
    } else if any.is::<PageCap>() {
        any.into(): PageCap;
    } else {
        panic!();
    }
}
