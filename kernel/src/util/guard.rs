use super::{MemoryObject};
use core::ops::{Deref, DerefMut, Index, IndexMut};

/// Read guard using a memory object.
pub struct UniqueReadGuard<T> {
    object: MemoryObject<T>
}

/// Write guard using a memory object.
pub struct UniqueWriteGuard<T> {
    object: MemoryObject<T>
}

// Implementation for UniqueReadGuard

impl<T> UniqueReadGuard<T> {
    /// Create a new read guard from a memory object.
    pub const unsafe fn new(object: MemoryObject<T>) -> Self {
        UniqueReadGuard::<T> {
            object: object,
        }
    }
}

unsafe impl<T> Send for UniqueReadGuard<T> { }
unsafe impl<T> Sync for UniqueReadGuard<T> { }

impl<T> Deref for UniqueReadGuard<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.object.as_ref() }
    }
}

// Implementation for UniqueWriteGuard

impl<T> UniqueWriteGuard<T> {
    /// Create a new write guard using a memory object.
    pub const unsafe fn new(object: MemoryObject<T>) -> Self {
        UniqueWriteGuard::<T> {
            object: object,
        }
    }
}

unsafe impl<T> Send for UniqueWriteGuard<T> { }
unsafe impl<T> Sync for UniqueWriteGuard<T> { }

impl<T> Deref for UniqueWriteGuard<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.object.as_ref() }
    }
}

impl<T> DerefMut for UniqueWriteGuard<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.object.as_mut() }
    }
}
