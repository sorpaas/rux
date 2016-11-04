mod untyped;
mod cpool;
mod task;

pub use self::untyped::{UntypedDescriptor, UntypedCap};
pub use self::cpool::{CPoolDescriptor, CPoolCap};
pub use self::task::{TaskDescriptor, TaskCap};
pub use arch::cap::{TopPageTableCap, PageCap, PAGE_LENGTH};

use arch;
use common::*;
use core::any::{TypeId};
use util::managed_arc::{ManagedWeakPool256Arc, ManagedArcAny, ManagedArc};

pub use abi::{SetDefault, TaskBuffer};
pub struct RawPage(pub [u8; PAGE_LENGTH]);
pub type RawPageCap = PageCap<RawPage>;
pub type TaskBufferPageCap = PageCap<TaskBuffer>;

impl SetDefault for RawPage {
    fn set_default(&mut self) {
        for raw in self.0.iter_mut() {
            *raw = 0x0;
        }
    }
}

pub unsafe fn upgrade_any(ptr: PAddr, type_id: TypeId) -> Option<ManagedArcAny> {
    if type_id == TypeId::of::<CPoolCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): CPoolCap }.into())
    } else if type_id == TypeId::of::<UntypedCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): UntypedCap }.into())
    } else if type_id == TypeId::of::<TaskCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): TaskCap }.into())
    } else if type_id == TypeId::of::<RawPageCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): RawPageCap }.into())
    } else if type_id == TypeId::of::<TaskBufferPageCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): TaskBufferPageCap }.into())
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
    } else if any.is::<RawPageCap>() {
        any.into(): RawPageCap;
    } else if any.is::<TaskBufferPageCap>() {
        any.into(): TaskBufferPageCap;
    } else {
        arch::cap::drop_any(any);
    }
}
