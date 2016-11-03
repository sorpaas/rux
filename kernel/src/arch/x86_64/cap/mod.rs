mod paging;

pub use self::paging::{PML4Descriptor, PML4Cap,
                       PDPTDescriptor, PDPTCap,
                       PDDescriptor, PDCap,
                       PTDescriptor, PTCap,
                       PageDescriptor, PageCap};

pub type TopPageTableCap = PML4Cap;

use common::*;
use core::any::{TypeId};
use util::managed_arc::{ManagedArc, ManagedWeakPool256Arc, ManagedArcAny};

pub unsafe fn upgrade_any(ptr: PAddr, type_id: TypeId) -> Option<ManagedArcAny> {
    if type_id == TypeId::of::<PML4Cap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): PML4Cap }.into())
    } else if type_id == TypeId::of::<PDPTCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): PDPTCap }.into())
    } else if type_id == TypeId::of::<PDCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): PDCap }.into())
    } else if type_id == TypeId::of::<PTCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): PTCap }.into())
    } else if type_id == TypeId::of::<PageCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): PageCap }.into())
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
