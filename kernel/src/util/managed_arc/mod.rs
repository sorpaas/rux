use core::any::{Any, TypeId};
use core::marker::PhantomData;
use core::convert::{From, Into};
use core::fmt;
use core::mem;
use core::ptr;
use common::*;
use spin::Mutex;
use util::MemoryObject;

/// Read/write lock for ManagedArc.
mod rwlock;
/// Weak pool storing weak pointers for ManagedArc.
mod weak_pool;

pub use self::rwlock::{ManagedArcRwLockReadGuard, ManagedArcRwLockWriteGuard};
pub use self::weak_pool::{ManagedWeakPool1Arc, ManagedWeakPool3Arc, ManagedWeakPool256Arc};

/// A weak node (entry of a weak pool).
#[derive(Debug)]
struct ManagedWeakNode {
    ptr: PAddr,
    strong_type_id: TypeId,
    prev: Option<ManagedWeakAddr>,
    next: Option<ManagedWeakAddr>
}

/// A weak address.
#[derive(Copy, Clone, Debug)]
struct ManagedWeakAddr {
    inner_addr: PAddr,
    inner_type_id: TypeId,
    offset: usize,
}

/// Inner of an Arc, containing strong pointers and weak pointers
/// information. Wrap the actual data.
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

/// A managed Arc, pointing to a `ManagedArcInner`.
pub struct ManagedArc<T> {
    ptr: PAddr,
    _marker: PhantomData<T>,
}

impl<T> fmt::Debug for ManagedArc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(0x{:x})", unsafe { ::core::intrinsics::type_name::<Self>() }, self.ptr)
    }
}

impl<T> Drop for ManagedArc<T> {
    fn drop(&mut self) {
        let mut inner_obj = self.inner_object();
        let inner = unsafe { inner_obj.as_mut() };
        let mut lead = inner.lead.lock();
        *lead -= 1;
    }
}

impl<T> Clone for ManagedArc<T> {
    fn clone(&self) -> Self {
        let mut inner_obj = self.inner_object();
        let inner = unsafe { inner_obj.as_mut() };
        let mut lead = inner.lead.lock();
        *lead += 1;

        ManagedArc {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

/// Like `ManagedArc<T>`, but use `TypeId` to represent its type.
#[derive(Debug)]
pub struct ManagedArcAny {
    ptr: PAddr,
    type_id: TypeId
}

impl ManagedArcAny {
    /// Check whether this Arc is of given type.
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
    /// Get the ManagedArcInner length.
    pub fn inner_length() -> usize {
        mem::size_of::<ManagedArcInner<T>>()
    }

    /// Get the ManagedArcInner alginment.
    pub fn inner_alignment() -> usize {
        mem::align_of::<ManagedArcInner<T>>()
    }

    /// Create a managed Arc from a physical address.
    pub unsafe fn from_ptr(ptr: PAddr) -> Self {
        let arc = ManagedArc { ptr: ptr, _marker: PhantomData };

        let inner_obj = arc.inner_object();
        let inner = unsafe { inner_obj.as_ref() };
        let mut lead = inner.lead.lock();
        *lead += 1;

        arc
    }

    /// Create a managed Arc using the given data.
    pub unsafe fn new(ptr: PAddr, data: T) -> Self {
        let arc = ManagedArc { ptr: ptr, _marker: PhantomData };
        let mut inner = arc.inner_object();
        ptr::write(inner.as_mut(), ManagedArcInner {
            lead: Mutex::new(1),
            first_weak: Mutex::new(None),
            data: data,
        });

        arc
    }

    /// Read the inner object, wrapped in a memory object.
    fn inner_object(&self) -> MemoryObject<ManagedArcInner<T>> {
        unsafe { MemoryObject::<ManagedArcInner<T>>::new(self.ptr) }
    }

    /// Get the strong pointers count.
    pub fn lead_count(&self) -> usize {
        let inner = self.inner_object();
        let lead = unsafe { inner.as_ref().lead.lock() };
        *lead
    }
}
