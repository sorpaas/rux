use core::any::{Any, TypeId};
use core::ops::{Index, IndexMut, Deref, DerefMut};
use core::marker::{PhantomData, Reflect};
use core::slice::{SliceExt};
use core::convert::{From, Into};
use core::fmt;
use core::mem;
use core::ptr;
use common::*;
use spin::{Mutex};
use util::{MemoryObject};

mod rwlock;
mod weak_pool;

pub use self::rwlock::{ManagedArcRwLockReadGuard, ManagedArcRwLockWriteGuard};
pub use self::weak_pool::{ManagedWeakPool1Arc, ManagedWeakPool2Arc, ManagedWeakPool256Arc};

#[derive(Debug)]
struct ManagedWeakNode {
    ptr: PAddr,
    strong_type_id: TypeId,
    prev: Option<ManagedWeakAddr>,
    next: Option<ManagedWeakAddr>
}

#[derive(Copy, Clone, Debug)]
struct ManagedWeakAddr {
    inner_addr: PAddr,
    inner_type_id: TypeId,
    offset: usize,
}

struct ManagedArcInner<T> {
    lead: Mutex<usize>,
    // TODO: Implement weak pool lock.
    first_weak: Mutex<Option<ManagedWeakAddr>>,
    data: T
}

impl<T> Drop for ManagedArcInner<T> {
    fn drop(&mut self) {
        let lead = self.lead.lock();
        assert!(*lead == 0);

        // TODO drop all weak pointers
        panic!();
    }
}

pub struct ManagedArc<T> {
    ptr: PAddr,
    _marker: PhantomData<T>,
}

impl<T> fmt::Debug for ManagedArc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ManagedArc {{ ptr: 0x{:x} }}", self.ptr)
    }
}

impl<T> Drop for ManagedArc<T> {
    fn drop(&mut self) {
        let inner_obj = self.inner_object();
        let inner = unsafe { inner_obj.as_mut().unwrap() };
        let mut lead = inner.lead.lock();
        *lead -= 1;
    }
}

impl<T> Clone for ManagedArc<T> {
    fn clone(&self) -> Self {
        let inner_obj = self.inner_object();
        let inner = unsafe { inner_obj.as_mut().unwrap() };
        let mut lead = inner.lead.lock();
        *lead += 1;

        ManagedArc {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct ManagedArcAny {
    ptr: PAddr,
    type_id: TypeId
}

impl ManagedArcAny {
    pub fn is<T: Any>(&self) -> bool
        where ManagedArc<T>: Any {
        self.type_id == TypeId::of::<T>()
    }
}

impl<T: Any> From<ManagedArcAny> for ManagedArc<T> {
    fn from(any: ManagedArcAny) -> Self {
        assert!(any.type_id == TypeId::of::<ManagedArc<T>>());
        let ptr = any.ptr;
        mem::forget(any);
        ManagedArc {
            ptr: ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: Any> Into<ManagedArcAny> for ManagedArc<T> {
    fn into(self) -> ManagedArcAny {
        let ptr = self.ptr;
        mem::forget(self);
        ManagedArcAny {
            ptr: ptr,
            type_id: TypeId::of::<ManagedArc<T>>(),
        }
    }
}

impl Drop for ManagedArcAny {
    fn drop(&mut self) {
        log!("Error: trying to drop a ManagedArcAny.");
        panic!();
    }
}

impl<T> ManagedArc<T> {
    pub fn inner_length() -> usize {
        mem::size_of::<ManagedArcInner<T>>()
    }

    pub fn inner_alignment() -> usize {
        mem::align_of::<ManagedArcInner<T>>()
    }

    pub unsafe fn from_ptr(ptr: PAddr) -> Self {
        let arc = ManagedArc { ptr: ptr, _marker: PhantomData };

        let inner_obj = arc.inner_object();
        let inner = unsafe { inner_obj.as_ref().unwrap() };
        let mut lead = inner.lead.lock();
        *lead += 1;

        arc
    }

    pub unsafe fn new(ptr: PAddr, data: T) -> Self {
        let arc = ManagedArc { ptr: ptr, _marker: PhantomData };
        let inner = arc.inner_object();
        ptr::write(inner.as_mut().unwrap(), ManagedArcInner {
            lead: Mutex::new(1),
            first_weak: Mutex::new(None),
            data: data,
        });

        arc
    }

    fn inner_object(&self) -> MemoryObject<ManagedArcInner<T>> {
        unsafe { MemoryObject::<ManagedArcInner<T>>::new(self.ptr) }
    }

    pub fn lead_count(&self) -> usize {
        let inner = self.inner_object();
        unsafe { *inner.as_ref().unwrap().lead.lock() }
    }
}
