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

            pub fn upgrade<T: Any>(&self, index: usize) -> Option<ManagedArc<T>>
                where ManagedArc<T>: Any {
                let inner_obj = self.inner_object();
                let inner = unsafe { inner_obj.as_ref().unwrap() };

                let upgrading_obj = inner.data[index].lock();
                let upgrading_weak = upgrading_obj.as_ref();

                upgrading_weak.and_then(|weak| {
                    if weak.strong_type_id != TypeId::of::<ManagedArc<T>>() {
                        None
                    } else {
                        let arc = ManagedArc::<T> {
                            ptr: weak.ptr,
                            _marker: PhantomData,
                        };
                        let arc_inner_obj = arc.inner_object();
                        let arc_inner = unsafe { arc_inner_obj.as_ref().unwrap() };
                        let mut lead = arc_inner.lead.lock();
                        *lead += 1;

                        Some(arc)
                    }
                })
            }

            pub fn downgrade_at<T: Any>(&self, arc: &ManagedArc<T>, index: usize)
                where ManagedArc<T>: Any {
                let inner_obj = self.inner_object();
                let inner = unsafe { inner_obj.as_ref().unwrap() };
                let ptr = inner.ptr;

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

                let mut weak_node_option = inner.data[index].lock();
                assert!(weak_node_option.is_none());

                let arc_inner_obj = arc.inner_object();
                let arc_inner = unsafe { inner_obj.as_ref().unwrap() };

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

            pub fn downgrade_free<T: Any>(&self, arc: &ManagedArc<T>) -> Option<usize>
                where ManagedArc<T>: Any {
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
weak_pool!(ManagedWeakPool2Arc, [Mutex<Option<ManagedWeakNode>>; 2]);
weak_pool!(ManagedWeakPool256Arc, [Mutex<Option<ManagedWeakNode>>; 256]);

fn set_weak_node<F>(addr: ManagedWeakAddr, f: F) where F: FnOnce(Option<ManagedWeakNode>) -> Option<ManagedWeakNode> {
    if addr.inner_type_id == TypeId::of::<ManagedArcInner<[Mutex<Option<ManagedWeakNode>>; 1]>>() {
        let inner_obj: MemoryObject<ManagedArcInner<[Mutex<Option<ManagedWeakNode>>; 1]>> =
            unsafe { MemoryObject::new(addr.inner_addr) };
        let inner = unsafe { inner_obj.as_ref().unwrap() };
        let mut weak_node = inner.data[addr.offset].lock();
        *weak_node = f((*weak_node).take());
    } else if addr.inner_type_id == TypeId::of::<ManagedArcInner<[Mutex<Option<ManagedWeakNode>>; 256]>>() {
        let inner_obj: MemoryObject<ManagedArcInner<[Mutex<Option<ManagedWeakNode>>; 256]>> =
            unsafe { MemoryObject::new(addr.inner_addr) };
        let inner = unsafe { inner_obj.as_ref().unwrap() };
        let mut weak_node = inner.data[addr.offset].lock();
        *weak_node = f((*weak_node).take());
    } else if addr.inner_type_id == TypeId::of::<ManagedArcInner<[Mutex<Option<ManagedWeakNode>>; 2]>>() {
        let inner_obj: MemoryObject<ManagedArcInner<[Mutex<Option<ManagedWeakNode>>; 2]>> =
            unsafe { MemoryObject::new(addr.inner_addr) };
        let inner = unsafe { inner_obj.as_ref().unwrap() };
        let mut weak_node = inner.data[addr.offset].lock();
        *weak_node = f((*weak_node).take());
    } else {
        panic!();
    }
}
