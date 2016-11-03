mod paging;

pub use self::paging::{PML4Descriptor, PML4Cap,
                       PDPTDescriptor, PDPTCap,
                       PDDescriptor, PDCap,
                       PTDescriptor, PTCap,
                       PageDescriptor, PageCap};

pub type TopPageTableCap = PML4Cap;

use util::managed_arc::{ManagedWeakPool256Arc, ManagedArcAny};

pub fn upgrade_any(weak_pool: &ManagedWeakPool256Arc, index: usize) -> Option<ManagedArcAny> {
    if let Some(r) = weak_pool.upgrade::<PML4Cap>(index) {
        Some(r.into())
    } else if let Some(r) = weak_pool.upgrade::<PDPTCap>(index) {
        Some(r.into())
    } else if let Some(r) = weak_pool.upgrade::<PDCap>(index) {
        Some(r.into())
    } else if let Some(r) = weak_pool.upgrade::<PTCap>(index) {
        Some(r.into())
    } else if let Some(r) = weak_pool.upgrade::<PageCap>(index) {
        Some(r.into())
    } else {
        None
    }
}
