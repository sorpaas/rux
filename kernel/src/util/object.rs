use common::*;
use core::ops::{Deref, DerefMut};
use core::cell::{UnsafeCell};
use core::marker::{PhantomData};

pub use spin::{ExternMutex, MutexGuard};
pub use arch::{MemoryObject};

pub struct ExternReadonlyObject<T> {
    pointer: UnsafeCell<Option<*const T>>,
    paddr: UnsafeCell<Option<PAddr>>,
    _marker: PhantomData<T>
}

impl<T> ExternReadonlyObject<T> {
    pub const unsafe fn new() -> Self {
        ExternReadonlyObject {
            pointer: UnsafeCell::new(None),
            paddr: UnsafeCell::new(None),
            _marker: PhantomData,
        }
    }

    pub unsafe fn bootstrap(&self, ptr: *const T, paddr: PAddr) {
        *self.pointer.get() = Some(ptr);
        *self.paddr.get() = Some(paddr);
    }

    pub unsafe fn unbootstrap(&self) {
        *self.pointer.get() = None;
        *self.paddr.get() = None;
    }

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
