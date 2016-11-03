mod untyped;
mod cpool;
mod task;

pub use self::untyped::{UntypedDescriptor, UntypedCap};
pub use self::cpool::{CPoolDescriptor, CPoolCap};
pub use self::task::{TaskDescriptor, TaskCap};
pub use arch::cap::{TopPageTableCap, PageCap};

use arch;
use common::*;
use core::any::{TypeId};
use util::managed_arc::{ManagedWeakPool256Arc, ManagedArcAny, ManagedArc};

pub unsafe fn upgrade_any(ptr: PAddr, type_id: TypeId) -> Option<ManagedArcAny> {
    if type_id == TypeId::of::<CPoolCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): CPoolCap }.into())
    } else if type_id == TypeId::of::<UntypedCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): UntypedCap }.into())
    } else if type_id == TypeId::of::<TaskCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): TaskCap }.into())
    } else {
        arch::cap::upgrade_any(ptr, type_id)
    }
}

pub fn drop_any(any: ManagedArcAny) {
    if any.is::<CPoolCap>() {
        any.into(): CPoolCap;
    } else if any.is::<UntypedCap>() {
        any.into(): UntypedCap;
    } else if any.is::<TaskCap>() {
        any.into(): TaskCap;
    } else {
        arch::cap::drop_any(any);
    }
}
