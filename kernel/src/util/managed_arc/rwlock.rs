use core::any::{Any, TypeId};
use core::ops::{Index, IndexMut, Deref, DerefMut};
use core::marker::{PhantomData};
use core::slice::{SliceExt};
use core::mem;
use core::ptr;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use common::*;
use util::{MemoryObject};

use super::{ManagedArcInner, ManagedArc};

/// A read guard for ManagedArc.
pub struct ManagedArcRwLockReadGuard<'a, T: 'a> {
    lock: RwLockReadGuard<'a, T>,
    object: MemoryObject<ManagedArcInner<RwLock<T>>>,
}

impl<'a, T: 'a> Deref for ManagedArcRwLockReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.lock.deref()
    }
}

/// A write guard for ManagedArc.
pub struct ManagedArcRwLockWriteGuard<'a, T: 'a> {
    lock: RwLockWriteGuard<'a, T>,
    object: MemoryObject<ManagedArcInner<RwLock<T>>>,
}

impl<'a, T: 'a> Deref for ManagedArcRwLockWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.lock.deref()
    }
}

impl<'a, T: 'a> DerefMut for ManagedArcRwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.lock.deref_mut()
    }
}

impl<U> ManagedArc<RwLock<U>> {
    /// Read the value from the ManagedArc. Returns the guard.
    pub fn read(&self) -> ManagedArcRwLockReadGuard<U> {
        let inner_obj = self.inner_object();
        let inner = unsafe { &*inner_obj.as_ptr() };
        ManagedArcRwLockReadGuard {
            lock: inner.data.read(),
            object: inner_obj
        }
    }

    /// Write to the ManagedArc. Returns the guard.
    pub fn write(&self) -> ManagedArcRwLockWriteGuard<U> {
        let inner_obj = self.inner_object();
        let inner = unsafe { &*inner_obj.as_ptr() };
        ManagedArcRwLockWriteGuard {
            lock: inner.data.write(),
            object: inner_obj
        }
    }
}
