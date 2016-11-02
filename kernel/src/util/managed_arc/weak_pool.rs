use core::any::{Any, TypeId};
use core::ops::{Index, IndexMut, Deref, DerefMut};
use core::marker::{PhantomData};
use core::slice::{SliceExt};
use core::mem;
use core::ptr;
use common::*;
use spin::{Mutex};
use util::{MemoryObject};

use super::{ManagedArc, ManagedArcInner, ManagedWeakAddr, ManagedWeakNode};

macro_rules! weak_pool {
    ( $name:ident, $t:ty ) => (
        pub type $name = ManagedArc<$t>;

        impl ManagedArc<$t> {
            pub unsafe fn create(ptr: PAddr) -> ManagedArc<$t> {
                let arc = ManagedArc::new(ptr, mem::uninitialized());
                let inner_obj = arc.inner_object();
                let inner = unsafe { inner_obj.as_mut().unwrap() };

                for (i, element) in (inner.data: $t).iter_mut().enumerate() {
                    ptr::write(element, Mutex::new(None));
                }

                arc
            }

            pub fn upgrade<T: Any>(&self, index: usize) -> Option<ManagedArc<T>> {
                let inner_obj = self.inner_object();
                let inner = unsafe { inner_obj.as_mut().unwrap() };
                let obj = inner.data[index].lock();

                obj.as_ref().map(|weak| {
                    assert!(weak.type_id == TypeId::of::<T>());
                    let arc = ManagedArc {
                        ptr: weak.ptr,
                        _marker: PhantomData,
                    };
                    let inner_obj = arc.inner_object();
                    let inner = unsafe { inner_obj.as_mut().unwrap() };
                    let mut lead = inner.lead.lock();
                    *lead += 1;

                    arc
                })
            }

            pub fn downgrade_at<T: Any>(&self, arc: &ManagedArc<T>, index: usize) {
                let self_inner_obj = self.inner_object();
                let self_inner = unsafe { self_inner_obj.as_mut().unwrap() };
                let self_ptr = self_inner.ptr;
                let weak_addr = ManagedWeakAddr {
                    pool_addr: self_ptr,
                    offset: index
                };
                let mut obj = self_inner.data[index].lock();
                assert!(obj.is_none());

                let mut node = ManagedWeakNode {
                    ptr: arc.ptr,
                    type_id: TypeId::of::<T>(),
                    prev: None,
                    next: None,
                };

                let inner_obj = arc.inner_object();
                let inner = unsafe { inner_obj.as_mut().unwrap() };
                let mut first_weak = inner.first_weak.lock();

                if first_weak.is_none() { // ArcInner doesn't have any weak.
                    *first_weak = Some(weak_addr);
                    *obj = Some(node);
                } else { // ArcInner has weak. Insert the new weak as the first child.
                    let second_weak_ptr = (*first_weak).unwrap();
                    let second_weak_node_inner_obj: MemoryObject<ManagedArcInner<[Mutex<Option<ManagedWeakNode>>; 256]>> = unsafe { MemoryObject::new(second_weak_ptr.pool_addr) };
                    let second_weak_node_inner = unsafe { second_weak_node_inner_obj.as_ref().unwrap() };
                    let ref second_weak_node = second_weak_node_inner.data[second_weak_ptr.offset];
                    let mut second_weak_option = second_weak_node.lock();
                    let second_weak = second_weak_option.as_mut().unwrap();
                    *first_weak = Some(weak_addr);
                    second_weak.prev = Some(weak_addr);
                    node.next = Some(second_weak_ptr);
                    *obj = Some(node);
                }
            }

            pub fn downgrade_free<'a, T: Any>(&self, arc: &ManagedArc<T>) -> Option<usize> {
                let inner_obj = self.inner_object();
                let inner = unsafe { inner_obj.as_mut().unwrap() };
                for (i, element) in inner.data.iter_mut().enumerate() {
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

weak_pool!(ManagedWeakPool1Arc, [Mutex<Option<ManagedWeakNode>>; 1]);
weak_pool!(ManagedWeakPool256Arc, [Mutex<Option<ManagedWeakNode>>; 256]);
