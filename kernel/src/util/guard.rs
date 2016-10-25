use super::{MemoryObject};
use core::ops::{Deref, DerefMut, Index, IndexMut};

pub use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct UniqueReadGuard<'a, T: 'a> {
    lock: RwLockReadGuard<'a, ()>,
    object: MemoryObject<T>
}

pub struct UniqueWriteGuard<'a, T: 'a> {
    lock: RwLockWriteGuard<'a, ()>,
    object: MemoryObject<T>
}

pub struct SharedReadGuard<'a, T: 'a> {
    lock: RwLockReadGuard<'a, T>,
    object: MemoryObject<RwLock<T>>
}

pub struct SharedWriteGuard<'a, T: 'a> {
    lock: RwLockWriteGuard<'a, T>,
    object: MemoryObject<RwLock<T>>
}

pub struct IndexedSharedReadGuard<'a, T: 'a, I: Copy, U: 'a + Index<I, Output=T>> {
    guard: SharedReadGuard<'a, U>,
    index: I,
}

pub struct IndexedSharedWriteGuard<'a, T: 'a, I: Copy, U: 'a + Index<I, Output=T> + IndexMut<I>> {
    guard: SharedWriteGuard<'a, U>,
    index: I,
}

// Implementation for UniqueReadGuard

impl<'a, T: 'a> UniqueReadGuard<'a, T> {
    pub const unsafe fn new(object: MemoryObject<T>, lock: RwLockReadGuard<'a, ()>) -> Self {
        UniqueReadGuard::<'a, T> {
            object: object,
            lock: lock
        }
    }
}

unsafe impl<'a, T: 'a> Send for UniqueReadGuard<'a, T> { }
unsafe impl<'a, T: 'a> Sync for UniqueReadGuard<'a, T> { }

impl<'a, T: 'a> Deref for UniqueReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.object.as_ref().unwrap() }
    }
}

// Implementation for UniqueWriteGuard

impl<'a, T: 'a> UniqueWriteGuard<'a, T> {
    pub const unsafe fn new(object: MemoryObject<T>, lock: RwLockWriteGuard<'a, ()>) -> Self {
        UniqueWriteGuard::<'a, T> {
            object: object,
            lock: lock
        }
    }
}

unsafe impl<'a, T: 'a> Send for UniqueWriteGuard<'a, T> { }
unsafe impl<'a, T: 'a> Sync for UniqueWriteGuard<'a, T> { }

impl<'a, T: 'a> Deref for UniqueWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.object.as_ref().unwrap() }
    }
}

impl<'a, T: 'a> DerefMut for UniqueWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.object.as_mut().unwrap() }
    }
}

// Implementation for SharedReadGuard

impl<'a, T: 'a> SharedReadGuard<'a, T> {
    pub unsafe fn new(object: MemoryObject<RwLock<T>>) -> Self {
        let rwlock = unsafe { object.as_ref().unwrap() };
        SharedReadGuard {
            object: object,
            lock: rwlock.read(),
        }
    }
}

unsafe impl<'a, T: 'a> Send for SharedReadGuard<'a, T> { }
unsafe impl<'a, T: 'a> Sync for SharedReadGuard<'a, T> { }

impl<'a, T: 'a> Deref for SharedReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.lock.deref()
    }
}

// Implementation for SharedWriteGuard

impl<'a, T: 'a> SharedWriteGuard<'a, T> {
    pub unsafe fn new(object: MemoryObject<RwLock<T>>) -> Self {
        let rwlock = unsafe { object.as_ref().unwrap() };
        SharedWriteGuard {
            object: object,
            lock: rwlock.write(),
        }
    }
}

unsafe impl<'a, T: 'a> Send for SharedWriteGuard<'a, T> { }
unsafe impl<'a, T: 'a> Sync for SharedWriteGuard<'a, T> { }

impl<'a, T: 'a> Deref for SharedWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.lock.deref()
    }
}

impl<'a, T: 'a> DerefMut for SharedWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.lock.deref_mut()
    }
}

// Implementation for IndexedSharedReadGuard

impl<'a, T: 'a, I: Copy, U: 'a + Index<I, Output=T>> IndexedSharedReadGuard<'a, T, I, U> {
    pub unsafe fn new(guard: SharedReadGuard<'a, U>, index: I) -> Self {
        IndexedSharedReadGuard {
            guard: guard,
            index: index,
        }
    }
}

unsafe impl<'a, T: 'a, I: Copy, U: 'a + Index<I, Output=T>> Send for IndexedSharedReadGuard<'a, T, I, U> { }
unsafe impl<'a, T: 'a, I: Copy, U: 'a + Index<I, Output=T>> Sync for IndexedSharedReadGuard<'a, T, I, U> { }

impl<'a, T: 'a, I: Copy, U: 'a + Index<I, Output=T>> Deref for IndexedSharedReadGuard<'a, T, I, U> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.guard[self.index]
    }
}

// Implementation for IndexedSharedWriteGuard

impl<'a, T: 'a, I: Copy, U: 'a + Index<I, Output=T> + IndexMut<I>> IndexedSharedWriteGuard<'a, T, I, U> {
    pub unsafe fn new(guard: SharedWriteGuard<'a, U>, index: I) -> Self {
        IndexedSharedWriteGuard {
            guard: guard,
            index: index,
        }
    }
}

unsafe impl<'a, T: 'a, I: Copy, U: 'a + Index<I, Output=T> + IndexMut<I>> Send for IndexedSharedWriteGuard<'a, T, I, U> { }
unsafe impl<'a, T: 'a, I: Copy, U: 'a + Index<I, Output=T> + IndexMut<I>> Sync for IndexedSharedWriteGuard<'a, T, I, U> { }

impl<'a, T: 'a, I: Copy, U: 'a + Index<I, Output=T> + IndexMut<I>> Deref for IndexedSharedWriteGuard<'a, T, I, U> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.guard[self.index]
    }
}

impl<'a, T: 'a, I: Copy, U: 'a + Index<I, Output=T> + IndexMut<I>> DerefMut for IndexedSharedWriteGuard<'a, T, I, U> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.guard[self.index]
    }
}
