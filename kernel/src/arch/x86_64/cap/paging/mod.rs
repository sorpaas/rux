mod page;
mod pml4;

pub use self::page::{PageHalf};
pub use self::pml4::{PML4Half};

use common::*;
use arch::paging::{BASE_PAGE_LENGTH,
                   PT, PTEntry, PT_P, PT_RW, PT_US,
                   PD, PDEntry, PD_P, PD_RW, PD_US,
                   PDPT, PDPTEntry, PDPT_P, PDPT_RW, PDPT_US};
use util::{MemoryObject, UniqueReadGuard, UniqueWriteGuard,
           RwLock, RwLockReadGuard, RwLockWriteGuard};
use cap::{UntypedHalf, Capability, CapReadRefObject};

macro_rules! paging_half {
    ( $t:ident, $sub_half: ty, $actual: ty, $entry: ident, $access: expr, $map_name: ident ) => {
        #[derive(Debug)]
        pub struct $t {
            start_paddr: PAddr,
            lock: RwLock<()>,
        }

        impl<'a> CapReadRefObject<'a, $actual, UniqueReadGuard<'a, $actual>> for $t {
            fn read(&'a self) -> UniqueReadGuard<'a, $actual> {
                unsafe { UniqueReadGuard::new(
                    MemoryObject::<$actual>::new(self.start_paddr),
                    self.lock.read()
                ) }
            }
        }

        impl $t {
            fn write(&mut self) -> UniqueWriteGuard<$actual> {
                unsafe { UniqueWriteGuard::new(
                    MemoryObject::<$actual>::new(self.start_paddr),
                    self.lock.write()
                ) }
            }

            pub fn start_paddr(&self) -> PAddr {
                self.start_paddr
            }

            pub fn length() -> usize {
                BASE_PAGE_LENGTH
            }

            pub fn new(untyped: &mut UntypedHalf) -> Self {
                let alignment = BASE_PAGE_LENGTH;
                let paddr = untyped.allocate(BASE_PAGE_LENGTH, alignment);

                let mut half = $t {
                    start_paddr: paddr,
                    lock: RwLock::new(()),
                };

                for entry in half.write().iter_mut() {
                    *entry = $entry::empty();
                }

                half
            }

            pub fn $map_name(&mut self, index: usize, sub: &mut $sub_half) {
                let mut current = self.write();
                assert!(!current[index].is_present());

                current[index] = $entry::new(sub.start_paddr(), $access);
            }
        }

    }
}

paging_half!(PTHalf, PageHalf, PT, PTEntry, PT_P | PT_RW | PT_US, map_page);
paging_half!(PDPTHalf, PDHalf, PDPT, PDPTEntry, PDPT_P | PDPT_RW | PDPT_US, map_pd);
paging_half!(PDHalf, PTHalf, PD, PDEntry, PD_P | PD_RW | PD_US, map_pt);
