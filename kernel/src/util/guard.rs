use super::{MemoryObject};
use core::ops::{Deref, DerefMut, Index, IndexMut};

pub struct UniqueReadGuard<T> {
    object: MemoryObject<T>
}

pub struct UniqueWriteGuard<T> {
    object: MemoryObject<T>
}

// Implementation for UniqueReadGuard

impl<T> UniqueReadGuard<T> {
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
        unsafe { self.object.as_ref().unwrap() }
    }
}

// Implementation for UniqueWriteGuard

impl<T> UniqueWriteGuard<T> {
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
        unsafe { self.object.as_ref().unwrap() }
    }
}

impl<T> DerefMut for UniqueWriteGuard<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.object.as_mut().unwrap() }
    }
}
