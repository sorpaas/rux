use common::{PAddr, VAddr};
use core::ops::{Deref, DerefMut};
use core::marker::{PhantomData};
use core::cell::{UnsafeCell};

pub use spin::{Mutex, MutexGuard, ExternMutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
pub use arch::{MemoryObject};

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

pub struct UniqueGuard<T, U: Deref<Target=*mut T>, L> {
    object: U,
    lock: L
}

unsafe impl<T, U: Deref<Target=*mut T>, L> Send for UniqueGuard<T, U, L> { }
unsafe impl<T, U: Deref<Target=*mut T>, L> Sync for UniqueGuard<T, U, L> { }

impl<T, U: Deref<Target=*mut T>, L> UniqueGuard<T, U, L> {
    /// Safety: must be sure that reference to the object is unique.
    pub const unsafe fn new(object: U, lock: L) -> Self {
        UniqueGuard {
            object: object,
            lock: lock,
        }
    }
}

impl<T, U: Deref<Target=*mut T>, L> Deref for UniqueGuard<T, U, L> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.object.as_ref().unwrap() }
    }
}

impl<T, U: Deref<Target=*mut T>, L> DerefMut for UniqueGuard<T, U, L> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.object.as_mut().unwrap() }
    }
}

pub type UniqueMemoryGuard<T, L> = UniqueGuard<T, MemoryObject<T>, L>;

pub struct ReadonlyGuard<T, U: Deref<Target=*mut T>, L> {
    object: U,
    lock: L
}

unsafe impl<T, U: Deref<Target=*mut T>, L> Send for ReadonlyGuard<T, U, L> { }
unsafe impl<T, U: Deref<Target=*mut T>, L> Sync for ReadonlyGuard<T, U, L> { }

impl<T, U: Deref<Target=*mut T>, L> ReadonlyGuard<T, U, L> {
    /// Safety: must be sure that reference to the object is unique.
    pub const unsafe fn new(object: U, lock: L) -> Self {
        ReadonlyGuard {
            object: object,
            lock: lock,
        }
    }
}

impl<T, U: Deref<Target=*mut T>, L> Deref for ReadonlyGuard<T, U, L> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.object.as_ref().unwrap() }
    }
}

pub type ReadonlyMemoryGuard<T, L> = ReadonlyGuard<T, MemoryObject<T>, L>;
