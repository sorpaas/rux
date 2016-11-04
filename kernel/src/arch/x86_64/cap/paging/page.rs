use common::*;
use arch::paging::{BASE_PAGE_LENGTH};
use util::{MemoryObject, UniqueReadGuard, UniqueWriteGuard, RwLock};
use util::managed_arc::{ManagedWeakPool1Arc};
use core::marker::{PhantomData};
use core::any::{Any};
use super::{PageDescriptor, PageCap};
use cap::{UntypedDescriptor, SetDefault};

pub const PAGE_LENGTH: usize = BASE_PAGE_LENGTH;

impl<T: SetDefault + Any> PageCap<T> {
    pub fn retype_from(untyped: &mut UntypedDescriptor) -> Self {
        let mut arc: Option<Self> = None;

        let start_paddr = unsafe { untyped.allocate(BASE_PAGE_LENGTH, BASE_PAGE_LENGTH) };

        let mapped_weak_pool = unsafe { ManagedWeakPool1Arc::create(
            untyped.allocate(ManagedWeakPool1Arc::inner_length(),
                             ManagedWeakPool1Arc::inner_alignment())) };

        unsafe {
            untyped.derive(Self::inner_length(), Self::inner_alignment(), |paddr, next_child| {
                let mut desc = PageDescriptor::<T> {
                    mapped_weak_pool: mapped_weak_pool,
                    start_paddr: start_paddr,
                    next: next_child,
                    _marker: PhantomData
                };

                desc.write().set_default();

                arc = Some(unsafe {
                    Self::new(paddr, RwLock::new(desc))
                });

                arc.clone().unwrap().into()
            });
        }

        arc.unwrap()
    }

    pub const fn length() -> usize {
        BASE_PAGE_LENGTH
    }
}

impl<T: SetDefault + Any> PageDescriptor<T> {
    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }

    pub fn length(&self) -> usize {
        BASE_PAGE_LENGTH
    }

    fn page_object(&self) -> MemoryObject<T> {
        unsafe { MemoryObject::new(self.start_paddr) }
    }

    pub fn read(&self) -> UniqueReadGuard<T> {
        unsafe { UniqueReadGuard::new(self.page_object()) }
    }

    pub fn write(&mut self) -> UniqueWriteGuard<T> {
        unsafe { UniqueWriteGuard::new(self.page_object()) }
    }
}
