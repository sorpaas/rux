use common::*;
use arch::paging::{BASE_PAGE_LENGTH};
use util::{MemoryObject, UniqueReadGuard, UniqueWriteGuard, RwLock};
use util::managed_arc::{ManagedWeakPool1Arc};
use super::{PageDescriptor, PageCap};
use cap::{UntypedDescriptor};

impl PageCap {
    pub fn retype_from(untyped: &mut UntypedDescriptor) -> Self {
        let mut arc: Option<Self> = None;

        let start_paddr = unsafe { untyped.allocate(BASE_PAGE_LENGTH, BASE_PAGE_LENGTH) };

        let mapped_weak_pool = unsafe { ManagedWeakPool1Arc::create(
            untyped.allocate(ManagedWeakPool1Arc::inner_length(),
                             ManagedWeakPool1Arc::inner_alignment())) };

        unsafe {
            untyped.derive(Self::inner_length(), Self::inner_alignment(), |paddr, next_child| {
                arc = Some(unsafe {
                    Self::new(paddr, RwLock::new(PageDescriptor {
                        mapped_weak_pool: mapped_weak_pool,
                        start_paddr: start_paddr,
                        next: next_child,
                    }))
                });

                arc.clone().unwrap().into()
            });
        }

        arc.unwrap()
    }
}

impl PageDescriptor {
    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }

    pub fn length(&self) -> usize {
        BASE_PAGE_LENGTH
    }

    fn page_object(&self) -> MemoryObject<[u8; BASE_PAGE_LENGTH]> {
        unsafe { MemoryObject::new(self.start_paddr) }
    }

    pub fn read(&self) -> UniqueReadGuard<[u8; BASE_PAGE_LENGTH]> {
        unsafe { UniqueReadGuard::new(self.page_object()) }
    }

    pub fn write(&mut self) -> UniqueWriteGuard<[u8; BASE_PAGE_LENGTH]> {
        unsafe { UniqueWriteGuard::new(self.page_object()) }
    }
}
