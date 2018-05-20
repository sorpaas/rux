macro_rules! doto_any {
    ($any:expr, $f:tt $(,$param:expr)*) => {
        if $any.is::<::cap::CPoolCap>() {
            $f ($any.into(): ::cap::CPoolCap, $($param),*)
        } else if $any.is::<::cap::UntypedCap>() {
            $f ($any.into(): ::cap::UntypedCap, $($param),*)
        } else if $any.is::<::cap::TaskCap>() {
            $f ($any.into(): ::cap::TaskCap, $($param),*)
        } else if $any.is::<::cap::RawPageCap>() {
            $f ($any.into(): ::cap::RawPageCap, $($param),*)
        } else if $any.is::<::cap::TaskBufferPageCap>() {
            $f ($any.into(): ::cap::TaskBufferPageCap, $($param),*)
        } else if $any.is::<::cap::ChannelCap>() {
            $f ($any.into(): ::cap::ChannelCap, $($param),*)
        } else {
            doto_arch_any!($any, $f $(,$param)*)
        }
    }
}

/// Untyped capability implementation.
mod untyped;
/// Capability pool capability implementation.
mod cpool;
/// Task capability implementation.
mod task;
/// Channel capability implementation.
mod channel;

pub use self::untyped::{UntypedDescriptor, UntypedCap};
pub use self::cpool::{CPoolDescriptor, CPoolCap};
pub use self::task::{TaskDescriptor, TaskCap, TaskStatus, idle, task_iter};
pub use self::channel::{ChannelDescriptor, ChannelCap, ChannelValue};

pub use arch::cap::{TopPageTableCap, PageCap, PAGE_LENGTH};

use arch;
use common::*;
use core::any::{TypeId};
use core::mem::drop;
use util::managed_arc::{ManagedArcAny, ManagedArc};

pub use abi::{SetDefault, TaskBuffer};
/// Raw page struct representing a whole page.
pub struct RawPage(pub [u8; PAGE_LENGTH]);
/// Raw page capability. Represents a page with no other information.
pub type RawPageCap = PageCap<RawPage>;
/// Task buffer page capability. Represents a page of task buffer.
pub type TaskBufferPageCap = PageCap<TaskBuffer>;

impl SetDefault for RawPage {
    fn set_default(&mut self) {
        for raw in self.0.iter_mut() {
            *raw = 0x0;
        }
    }
}

/// Create a managed Arc (capability) from an address of an kernel
/// object (architecture-specific or general). The `type_id` should be
/// a [TypeId](https://doc.rust-lang.org/std/any/struct.TypeId.html)
/// of a capability. If the `type_id` is not recognized, `None` is
/// returned.
///
/// # Safety
///
/// `ptr` must be a physical address pointing to a valid kernel object
/// of type `type_id`.
pub unsafe fn upgrade_any(ptr: PAddr, type_id: TypeId) -> Option<ManagedArcAny> {
    if type_id == TypeId::of::<CPoolCap>() {
        Some({ ManagedArc::from_ptr(ptr): CPoolCap }.into())
    } else if type_id == TypeId::of::<UntypedCap>() {
        Some({ ManagedArc::from_ptr(ptr): UntypedCap }.into())
    } else if type_id == TypeId::of::<TaskCap>() {
        Some({ ManagedArc::from_ptr(ptr): TaskCap }.into())
    } else if type_id == TypeId::of::<RawPageCap>() {
        Some({ ManagedArc::from_ptr(ptr): RawPageCap }.into())
    } else if type_id == TypeId::of::<TaskBufferPageCap>() {
        Some({ ManagedArc::from_ptr(ptr): TaskBufferPageCap }.into())
    } else if type_id == TypeId::of::<ChannelCap>() {
        Some({ ManagedArc::from_ptr(ptr): ChannelCap }.into())
    } else {
        arch::cap::upgrade_arch_any(ptr, type_id)
    }
}

/// Drop an architecture-specific `any` capability. `ManagedArcAny` is
/// not itself droppable. It must be converted to its real type before
/// dropping.
pub fn drop_any(any: ManagedArcAny) {
    doto_any!(any, drop)
}
