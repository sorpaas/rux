use core::any::{Any, TypeId};
use core::ops::{Index, IndexMut, Deref, DerefMut};
use core::marker::{PhantomData};
use core::slice::{SliceExt};
use core::mem;
use core::ptr;
use common::*;
use spin::{Mutex};
use util::{MemoryObject};

use super::{ManagedArc, ManagedArcAny, ManagedArcInner, ManagedWeakAddr, ManagedWeakNode};

/// Managed weak pool of size 1.
pub struct ManagedWeakPool1([Mutex<Option<ManagedWeakNode>>; 1], PAddr);
/// Managed weak pool of size 3.
pub struct ManagedWeakPool3([Mutex<Option<ManagedWeakNode>>; 3], PAddr);
/// Managed weak pool of size 256.
pub struct ManagedWeakPool256([Mutex<Option<ManagedWeakNode>>; 256], PAddr);

/// Managed Arc for weak pool of size 1.
pub type ManagedWeakPool1Arc = ManagedArc<ManagedWeakPool1>;
/// Managed Arc for weak pool of size 3.
pub type ManagedWeakPool3Arc = ManagedArc<ManagedWeakPool3>;
/// Managed Arc for weak pool of size 256.
pub type ManagedWeakPool256Arc = ManagedArc<ManagedWeakPool256>;

/// Guard for managed weak pool.
pub struct ManagedWeakPoolGuard<T> {
    object: MemoryObject<ManagedArcInner<T>>
}

impl<T> Deref for ManagedWeakPoolGuard<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &self.object.as_ref().unwrap().data }
    }
}

macro_rules! weak_pool {
    ( $t:ty ) => (
        impl ManagedArc<$t> {
            /// Create a managed weak pool in the given physical address.
            pub unsafe fn create(ptr: PAddr) -> Self {
                let arc = ManagedArc::new(ptr, mem::uninitialized());
                let inner_obj = arc.inner_object();
                let inner: &mut ManagedArcInner<$t> = unsafe { inner_obj.as_mut().unwrap() };

                ptr::write(&mut inner.data.1, ptr);
                for (i, element) in (inner.data: $t).0.iter_mut().enumerate() {
                    ptr::write(element, Mutex::new(None));
                }

                arc
            }

            /// Read the Arc. Returns the guard.
            pub fn read(&self) -> ManagedWeakPoolGuard<$t> {
                ManagedWeakPoolGuard { object: self.inner_object() }
            }
        }

        impl $t {
            /// Create a new strong pointer if `index` points to a
            /// non-none weak pointer in the weak pool.
            pub unsafe fn upgrade_any<F>(&self, index: usize, f: F) -> Option<ManagedArcAny> where F: FnOnce(PAddr, TypeId) -> Option<ManagedArcAny> {
                let upgrading_obj = self.0[index].lock();
                let upgrading_weak = upgrading_obj.as_ref();

                upgrading_weak.and_then(|weak| {
                    f(weak.ptr, weak.strong_type_id)
                })
            }

            /// Like `upgrade_any`, but create the pointer using the
            /// given type.
            pub fn upgrade<T: Any>(&self, index: usize) -> Option<ManagedArc<T>>
                where ManagedArc<T>: Any {
                let upgrading_obj = self.0[index].lock();
                let upgrading_weak = upgrading_obj.as_ref();

                upgrading_weak.and_then(|weak| {
                    if weak.strong_type_id != TypeId::of::<ManagedArc<T>>() {
                        None
                    } else {
                        Some(unsafe { ManagedArc::<T>::from_ptr(weak.ptr) })
                    }
                })
            }

            /// Downgrade a strong pointer to a weak pointer and store
            /// it at `index` in this weak pool.
            pub fn downgrade_at<T: Any>(&self, arc: &ManagedArc<T>, index: usize)
                where ManagedArc<T>: Any {

                let ptr = self.1;

                let weak_addr = ManagedWeakAddr {
                    inner_addr: ptr,
                    offset: index,
                    inner_type_id: TypeId::of::<ManagedArcInner<$t>>()
                };
                let mut weak_node = ManagedWeakNode {
                    ptr: arc.ptr,
                    strong_type_id: TypeId::of::<ManagedArc<T>>(),
                    prev: None,
                    next: None
                };

                let mut weak_node_option = self.0[index].lock();
                assert!(weak_node_option.is_none());

                let arc_inner_obj = arc.inner_object();
                let arc_inner = unsafe { arc_inner_obj.as_ref().unwrap() };

                let mut arc_first_weak = arc_inner.first_weak.lock();

                if arc_first_weak.is_none() {
                    // ArcInner doesn't have any weak.
                    *arc_first_weak = Some(weak_addr);
                    *weak_node_option = Some(weak_node);

                } else {
                    // ArcInner has weak. Insert the new weak as the first child.
                    let arc_second_weak_addr = arc_first_weak.take().unwrap();
                    set_weak_node(arc_second_weak_addr, |mut second_weak_node| {
                        assert!(second_weak_node.is_some());

                        second_weak_node.map(|mut second_weak_node| {
                            second_weak_node.prev = Some(weak_addr);
                            second_weak_node
                        })
                    });
                    weak_node.next = Some(arc_second_weak_addr);

                    *arc_first_weak = Some(weak_addr);
                    *weak_node_option = Some(weak_node);
                }
            }

            /// Downgrade a strong pointer to a weak pointer, and then
            /// store it in a free slot in this weak pool.
            pub fn downgrade_free<T: Any>(&self, arc: &ManagedArc<T>) -> Option<usize>
                where ManagedArc<T>: Any {
                for (i, element) in self.0.iter().enumerate() {
                    // TODO race conditions

                    if { element.lock().is_none() } {
                        self.downgrade_at(arc, i);
                        return Some(i);
                    }
                }
                None
            }
        }
    )
}

weak_pool!(ManagedWeakPool1);
weak_pool!(ManagedWeakPool3);
weak_pool!(ManagedWeakPool256);

fn set_weak_node<F>(addr: ManagedWeakAddr, f: F) where F: FnOnce(Option<ManagedWeakNode>) -> Option<ManagedWeakNode> {
    if addr.inner_type_id == TypeId::of::<ManagedArcInner<ManagedWeakPool1>>() {
        let inner_obj: MemoryObject<ManagedArcInner<ManagedWeakPool1>> =
            unsafe { MemoryObject::new(addr.inner_addr) };
        let inner = unsafe { inner_obj.as_ref().unwrap() };
        let mut weak_node = inner.data.0[addr.offset].lock();
        *weak_node = f((*weak_node).take());
    } else if addr.inner_type_id == TypeId::of::<ManagedArcInner<ManagedWeakPool256>>() {
        let inner_obj: MemoryObject<ManagedArcInner<ManagedWeakPool256>> =
            unsafe { MemoryObject::new(addr.inner_addr) };
        let inner = unsafe { inner_obj.as_ref().unwrap() };
        let mut weak_node = inner.data.0[addr.offset].lock();
        *weak_node = f((*weak_node).take());
    } else if addr.inner_type_id == TypeId::of::<ManagedArcInner<ManagedWeakPool3>>() {
        let inner_obj: MemoryObject<ManagedArcInner<ManagedWeakPool3>> =
            unsafe { MemoryObject::new(addr.inner_addr) };
        let inner = unsafe { inner_obj.as_ref().unwrap() };
        let mut weak_node = inner.data.0[addr.offset].lock();
        *weak_node = f((*weak_node).take());
    } else {
        panic!();
    }
}
