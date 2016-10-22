use common::{PAddr, VAddr};
use core::ops::{Deref};
use core::marker::{PhantomData};
use core::cell::{UnsafeCell};

pub fn align_up(paddr: PAddr, alignment: usize) -> PAddr {
    let raw = paddr.into(): usize;
    let aligned = if raw % alignment == 0 {
        raw
    } else {
        raw + (alignment - (raw % alignment))
    };
    PAddr::from(aligned)
}

pub fn align_down(paddr: PAddr, alignment: usize) -> PAddr {
    let raw = paddr.into(): usize;
    let aligned = if raw % alignment == 0 {
        raw
    } else {
        raw - (raw % alignment)
    };
    PAddr::from(aligned)
}

pub fn block_count(length: usize, block_length: usize) -> usize {
    if length % block_length == 0 {
        length / block_length
    } else {
        length / block_length + 1
    }
}

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
