macro_rules! doto_arch_any {
    ($any:expr, $f:tt $(,$param:expr)*) => {
        if $any.is::<::arch::cap::PML4Cap>() {
            $f ($any.into(): ::arch::cap::PML4Cap, $($param),*)
        } else if $any.is::<::arch::cap::PDPTCap>() {
            $f ($any.into(): ::arch::cap::PDPTCap, $($param),*)
        } else if $any.is::<::arch::cap::PDCap>() {
            $f ($any.into(): ::arch::cap::PDCap, $($param),*)
        } else if $any.is::<::arch::cap::PTCap>() {
            $f ($any.into(): ::arch::cap::PTCap, $($param),*)
        } else {
            panic!();
        }
    }
}

/// Paging-related arch-specific capabilities.
mod paging;

pub use self::paging::{PML4Descriptor, PML4Cap,
                       PDPTDescriptor, PDPTCap,
                       PDDescriptor, PDCap,
                       PTDescriptor, PTCap,
                       PageDescriptor, PageCap,
                       PAGE_LENGTH};

/// The top-level page table capability. In `x86_64`, this is PML4.
pub type TopPageTableCap = PML4Cap;

use common::*;
use core::any::{TypeId};
use util::managed_arc::{ManagedArc, ManagedWeakPool256Arc, ManagedArcAny};

/// Create a managed Arc (capability) from an address of an
/// architecture-specific kernel object. The `type_id` should be a
/// [TypeId](https://doc.rust-lang.org/std/any/struct.TypeId.html) of
/// an architecture-specific capability. If the `type_id` is not
/// recognized, `None` is returned.
///
/// # Safety
///
/// `ptr` must be a physical address pointing to a valid kernel object
/// of type `type_id`.
pub unsafe fn upgrade_arch_any(ptr: PAddr, type_id: TypeId) -> Option<ManagedArcAny> {
    if type_id == TypeId::of::<PML4Cap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): PML4Cap }.into())
    } else if type_id == TypeId::of::<PDPTCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): PDPTCap }.into())
    } else if type_id == TypeId::of::<PDCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): PDCap }.into())
    } else if type_id == TypeId::of::<PTCap>() {
        Some(unsafe { ManagedArc::from_ptr(ptr): PTCap }.into())
    } else {
        None
    }
}

/// Drop an architecture-specific `any` capability. `ManagedArcAny` is
/// not itself droppable. It must be converted to its real type before
/// dropping. This function is used by `kernel::cap::drop_any`.
pub fn drop_any(any: ManagedArcAny) {
    if any.is::<PML4Cap>() {
        any.into(): PML4Cap;
    } else if any.is::<PDPTCap>() {
        any.into(): PDPTCap;
    } else if any.is::<PDCap>() {
        any.into(): PDCap;
    } else if any.is::<PTCap>() {
        any.into(): PTCap;
    } else {
        panic!();
    }
}
