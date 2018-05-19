use common::*;
use core::ops::Deref;
use core::cell::{UnsafeCell};
use core::marker::{PhantomData};

pub use spin::{ExternMutex, MutexGuard};
pub use arch::{MemoryObject};

/// Represents an external readonly object.
pub struct ExternReadonlyObject<T> {
    pointer: UnsafeCell<Option<*const T>>,
    paddr: UnsafeCell<Option<PAddr>>,
    _marker: PhantomData<T>
}

impl<T> ExternReadonlyObject<T> {
    /// Create a new object that doesn't point to anything.
    pub const unsafe fn new() -> Self {
        ExternReadonlyObject {
            pointer: UnsafeCell::new(None),
            paddr: UnsafeCell::new(None),
            _marker: PhantomData,
        }
    }

    /// Bootstrap the pointer using a physical address.
    pub unsafe fn bootstrap(&self, ptr: *const T, paddr: PAddr) {
        *self.pointer.get() = Some(ptr);
        *self.paddr.get() = Some(paddr);
    }

    /// Unbootstrap the pointer. Erase the address stored.
    pub unsafe fn unbootstrap(&self) {
        *self.pointer.get() = None;
        *self.paddr.get() = None;
    }

    /// Get the physical address in the object.
    pub fn paddr(&self) -> PAddr {
        unsafe { (*self.paddr.get()).unwrap() }
    }
}

unsafe impl<T> Send for ExternReadonlyObject<T> { }
unsafe impl<T> Sync for ExternReadonlyObject<T> { }

impl<T> Deref for ExternReadonlyObject<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*((*self.pointer.get()).unwrap()) }
    }
}
