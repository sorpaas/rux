mod page;
mod pml4;

use common::*;
use arch::paging::{BASE_PAGE_LENGTH,
                   PT, PTEntry, PT_P, PT_RW, PT_US,
                   PD, PDEntry, PD_P, PD_RW, PD_US,
                   PDPT, PDPTEntry, PDPT_P, PDPT_RW, PDPT_US};
use util::{MemoryObject, UniqueReadGuard, UniqueWriteGuard, RwLock};
use util::managed_arc::{ManagedArc, ManagedArcAny, ManagedWeakPool1Arc};
use core::marker::{PhantomData};
use core::any::{Any};
use cap::{UntypedDescriptor, SetDefault};

pub use self::page::{PAGE_LENGTH};

pub struct PML4Descriptor {
    start_paddr: PAddr,
    next: Option<ManagedArcAny>,
}
pub type PML4Cap = ManagedArc<RwLock<PML4Descriptor>>;

pub struct PDPTDescriptor {
    mapped_weak_pool: ManagedWeakPool1Arc,
    start_paddr: PAddr,
    next: Option<ManagedArcAny>,
}
pub type PDPTCap = ManagedArc<RwLock<PDPTDescriptor>>;

pub struct PDDescriptor {
    mapped_weak_pool: ManagedWeakPool1Arc,
    start_paddr: PAddr,
    next: Option<ManagedArcAny>,
}
pub type PDCap = ManagedArc<RwLock<PDDescriptor>>;

pub struct PTDescriptor {
    mapped_weak_pool: ManagedWeakPool1Arc,
    start_paddr: PAddr,
    next: Option<ManagedArcAny>,
}
pub type PTCap = ManagedArc<RwLock<PTDescriptor>>;

pub struct PageDescriptor<T: SetDefault + Any> {
    mapped_weak_pool: ManagedWeakPool1Arc,
    start_paddr: PAddr,
    next: Option<ManagedArcAny>,
    _marker: PhantomData<T>
}
pub type PageCap<T: SetDefault + Any> = ManagedArc<RwLock<PageDescriptor<T>>>;

macro_rules! paging_cap {
    ( $cap:ty, $desc:tt, $paging:ty, $entry:tt, $map_fn:ident, $sub_cap:ty, $access:expr ) => (
        impl $cap {
            pub fn retype_from(untyped: &mut UntypedDescriptor) -> Self {
                let mut arc: Option<Self> = None;

                let start_paddr = unsafe { untyped.allocate(BASE_PAGE_LENGTH, BASE_PAGE_LENGTH) };

                let mapped_weak_pool = unsafe { ManagedWeakPool1Arc::create(
                    untyped.allocate(ManagedWeakPool1Arc::inner_length(),
                                     ManagedWeakPool1Arc::inner_alignment())) };

                unsafe {
                    untyped.derive(Self::inner_length(), Self::inner_alignment(), |paddr, next_child| {
                        let mut desc = $desc {
                            mapped_weak_pool: mapped_weak_pool,
                            start_paddr: start_paddr,
                            next: next_child,
                        };

                        for item in desc.write().iter_mut() {
                            *item = $entry::empty();
                        }

                        arc = Some(unsafe {
                            Self::new(paddr, RwLock::new(desc))
                        });

                        arc.clone().unwrap().into()
                    });
                }

                arc.unwrap()
            }

            pub fn $map_fn(&mut self, index: usize, sub: &$sub_cap) {
                let mut current_desc = self.write();
                let mut current = current_desc.write();
                let sub_desc = sub.read();
                assert!(!current[index].is_present());

                sub_desc.mapped_weak_pool.read().downgrade_at(self, 0);
                current[index] = $entry::new(sub_desc.start_paddr(), $access);
            }
        }

        impl $desc {
            pub fn start_paddr(&self) -> PAddr {
                self.start_paddr
            }

            pub fn length(&self) -> usize {
                BASE_PAGE_LENGTH
            }

            fn page_object(&self) -> MemoryObject<$paging> {
                unsafe { MemoryObject::new(self.start_paddr) }
            }

            pub fn read(&self) -> UniqueReadGuard<$paging> {
                unsafe { UniqueReadGuard::new(self.page_object()) }
            }

            fn write(&mut self) -> UniqueWriteGuard<$paging> {
                unsafe { UniqueWriteGuard::new(self.page_object()) }
            }
        }
    )
}

paging_cap!(PDPTCap, PDPTDescriptor, PDPT, PDPTEntry, map_pd, PDCap, PDPT_P | PDPT_RW | PDPT_US);
paging_cap!(PDCap, PDDescriptor, PD, PDEntry, map_pt, PTCap, PD_P | PD_RW | PD_US);

impl PTCap {
    pub fn retype_from(untyped: &mut UntypedDescriptor) -> Self {
        let mut arc: Option<Self> = None;

        let start_paddr = unsafe { untyped.allocate(BASE_PAGE_LENGTH, BASE_PAGE_LENGTH) };

        let mapped_weak_pool = unsafe { ManagedWeakPool1Arc::create(
            untyped.allocate(ManagedWeakPool1Arc::inner_length(),
                             ManagedWeakPool1Arc::inner_alignment())) };

        unsafe {
            untyped.derive(Self::inner_length(), Self::inner_alignment(), |paddr, next_child| {
                let mut desc = PTDescriptor {
                    mapped_weak_pool: mapped_weak_pool,
                    start_paddr: start_paddr,
                    next: next_child,
                };

                for item in desc.write().iter_mut() {
                    *item = PTEntry::empty();
                }

                arc = Some(unsafe {
                    Self::new(paddr, RwLock::new(desc))
                });

                arc.clone().unwrap().into()
            });
        }

        arc.unwrap()
    }

    pub fn map_page<T: SetDefault + Any>(&mut self, index: usize, sub: &PageCap<T>) {
        let mut current_desc = self.write();
        let mut current = current_desc.write();
        let sub_desc = sub.read();
        assert!(!current[index].is_present());

        sub_desc.mapped_weak_pool.read().downgrade_at(self, 0);
        current[index] = PTEntry::new(sub_desc.start_paddr(), PT_P | PT_RW | PT_US);
    }
}

impl PTDescriptor {
    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }

    pub fn length(&self) -> usize {
        BASE_PAGE_LENGTH
    }

    fn page_object(&self) -> MemoryObject<PT> {
        unsafe { MemoryObject::new(self.start_paddr) }
    }

    pub fn read(&self) -> UniqueReadGuard<PT> {
        unsafe { UniqueReadGuard::new(self.page_object()) }
    }

    fn write(&mut self) -> UniqueWriteGuard<PT> {
        unsafe { UniqueWriteGuard::new(self.page_object()) }
    }
}
