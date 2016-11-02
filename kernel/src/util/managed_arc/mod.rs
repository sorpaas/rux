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
pub use self::weak_pool::{ManagedWeakPool1Arc, ManagedWeakPool256Arc};

struct ManagedWeakNode {
    ptr: PAddr,
    type_id: TypeId,
    prev: Option<ManagedWeakAddr>,
    next: Option<ManagedWeakAddr>
}

#[derive(Copy, Clone, Debug)]
struct ManagedWeakAddr {
    pool_addr: PAddr,
    offset: usize,
}

struct ManagedArcInner<T> {
    lead: Mutex<usize>,
    // TODO: Implement weak pool lock.
    first_weak: Mutex<Option<ManagedWeakAddr>>,
    ptr: PAddr, // A pointer to self
    data: T
}

impl<T> Drop for ManagedArcInner<T> {
    fn drop(&mut self) {
        let lead = self.lead.lock();
        assert!(*lead == 0);

        let mut next_weak_ptr_option = *self.first_weak.lock();
        while next_weak_ptr_option.is_some() {
            let next_weak_ptr = next_weak_ptr_option.take().unwrap();
            let next_weak_obj = unsafe { MemoryObject::<Mutex<Option<ManagedWeakNode>>>::new(next_weak_ptr.pool_addr + next_weak_ptr.offset * mem::size_of::<ManagedWeakNode>()) };
            let mut next_weak_node = unsafe { next_weak_obj.as_mut().unwrap().lock() };
            next_weak_ptr_option = next_weak_node.as_ref().map(|node| { node.next }).unwrap();
            *next_weak_node = None;
        }
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

impl<T: Reflect + 'static> From<ManagedArcAny> for ManagedArc<T> {
    fn from(any: ManagedArcAny) -> Self {
        assert!(any.type_id == TypeId::of::<T>());
        let ptr = any.ptr;
        mem::forget(any);
        ManagedArc {
            ptr: ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: Reflect + 'static> Into<ManagedArcAny> for ManagedArc<T> {
    fn into(self) -> ManagedArcAny {
        let ptr = self.ptr;
        mem::forget(self);
        ManagedArcAny {
            ptr: ptr,
            type_id: TypeId::of::<T>(),
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

    pub unsafe fn new(ptr: PAddr, data: T) -> Self {
        log!("new called");
        let arc = ManagedArc { ptr: ptr, _marker: PhantomData };
        let inner = arc.inner_object();
        log!("got inner");
        ptr::write(inner.as_mut().unwrap(), ManagedArcInner {
            ptr: ptr,
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
