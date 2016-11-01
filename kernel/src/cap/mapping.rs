use core::any::{Any, TypeId};
use core::ops::{Index, IndexMut};
use core::marker::{PhantomData};
use core::slice::{SliceExt};
use core::mem;
use core::ptr;
use common::*;
use util::{Mutex, MemoryObject};

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

impl<T> Drop for ManagedArc<T> {
    fn drop(&mut self) {
        let inner_obj = self.inner_object();
        let inner = unsafe { inner_obj.as_mut().unwrap() };
        let mut lead = inner.lead.lock();
        *lead -= 1;
    }
}

pub struct ManagedWeakNode {
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

pub struct ManagedWeakPool<U: Index<usize, Output=Mutex<Option<ManagedWeakNode>>> + IndexMut<usize>>(U);
pub type ManagedWeakPool1 = ManagedWeakPool<[ManagedWeakNode; 1]>;
pub type ManagedWeakPool256 = ManagedWeakPool<[ManagedWeakNode; 256]>;

impl<T> ManagedArc<T> {
    pub fn inner_length() -> usize {
        mem::size_of::<ManagedArcInner<T>>()
    }

    pub unsafe fn new(ptr: PAddr, data: T) -> ManagedArc<T> {
        let arc = ManagedArc { ptr: ptr, _marker: PhantomData };
        let inner = arc.inner_object();
        *(inner.as_mut().unwrap()) = ManagedArcInner {
            ptr: ptr,
            lead: Mutex::new(1),
            first_weak: Mutex::new(None),
            data: data,
        };

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

impl<U: Index<usize, Output=Mutex<Option<ManagedWeakNode>>> + IndexMut<usize>> ManagedWeakPool<U> {
    pub fn inner_length() -> usize {
        mem::size_of::<ManagedArcInner<ManagedWeakPool<U>>>()
    }

    pub unsafe fn new<'a>(ptr: PAddr) -> ManagedArc<ManagedWeakPool<U>>
        where U: SliceExt<Item=Mutex<Option<ManagedWeakNode>>> {
        let arc = ManagedArc::<ManagedWeakPool<U>>::new(ptr, mem::uninitialized());
        let inner = arc.inner_object();

        for (i, element) in ((*(inner.as_mut().unwrap())).data: ManagedWeakPool<U>).0.iter_mut().enumerate() {
            ptr::write(element, Mutex::new(None));
        }

        arc
    }

    pub fn upgrade<T: Any>(&self, index: usize) -> Option<ManagedArc<T>> {
        let obj = self.0[index].lock();

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
}

impl<U: Index<usize, Output=Mutex<Option<ManagedWeakNode>>> + IndexMut<usize>> ManagedArc<ManagedWeakPool<U>> {
    pub fn downgrade_at<T: Any>(&self, arc: &ManagedArc<T>, index: usize) {
        let self_inner_obj = self.inner_object();
        let self_inner = unsafe { self_inner_obj.as_mut().unwrap() };
        let self_ptr = self_inner.ptr;
        let weak_addr = ManagedWeakAddr {
            pool_addr: self_ptr,
            offset: index
        };
        let mut obj = self_inner.data.0[index].lock();
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
            let second_weak_node_obj = unsafe { MemoryObject::<Mutex<Option<ManagedWeakNode>>>::new(second_weak_ptr.pool_addr + second_weak_ptr.offset * mem::size_of::<ManagedWeakNode>()) };
            let second_weak_node = unsafe { second_weak_node_obj.as_ref().unwrap() };
            let mut second_weak_option = second_weak_node.lock();
            let second_weak = second_weak_option.as_mut().unwrap();
            *first_weak = Some(weak_addr);
            second_weak.prev = Some(weak_addr);
            node.next = Some(second_weak_ptr);
            *obj = Some(node);
        }
    }

    pub fn downgrade_free<T: Any>(&self, arc: &ManagedArc<T>) -> Option<usize>
        where U: SliceExt<Item=Mutex<Option<ManagedWeakNode>>> {
        for (i, element) in (*(unsafe { self.inner_object().as_mut() }.unwrap())).data.0.iter_mut().enumerate() {
            // TODO race conditions

            if { element.lock().is_none() } {
                self.downgrade_at(arc, i);
                return Some(i);
            }
        }
        None
    }
}
