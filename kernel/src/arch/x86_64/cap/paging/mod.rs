mod page;
mod pml4;

pub use self::page::{PageHalf};
pub use self::pml4::{PML4Half};

use common::*;
use arch::paging::{BASE_PAGE_LENGTH,
                   PT, PTEntry, PT_P, PT_RW, PT_US,
                   PD, PDEntry, PD_P, PD_RW, PD_US,
                   PDPT, PDPTEntry, PDPT_P, PDPT_RW, PDPT_US};
use utils::{MemoryObject, UniqueMemoryGuard, ReadonlyMemoryGuard,
            RwLock, RwLockReadGuard, RwLockWriteGuard};
use cap::{UntypedHalf, Capability, CapReadonlyObject, CapHalf};

macro_rules! paging_half {
    ( $t:ident, $sub_half: ty, $actual: ty, $entry: ident, $access: expr, $map_name: ident ) => {
        #[derive(Debug)]
        pub struct $t {
            start_paddr: PAddr,
            lock: RwLock<()>,
            deleted: bool
        }

        normal_half!($t);

        impl<'a> CapReadonlyObject<'a, $actual, ReadonlyMemoryGuard<$actual, RwLockReadGuard<'a, ()>>> for $t {
            fn lock(&self) -> ReadonlyMemoryGuard<$actual, RwLockReadGuard<()>> {
                unsafe { ReadonlyMemoryGuard::new(
                    MemoryObject::<$actual>::new(self.start_paddr),
                    self.lock.read()
                ) }
            }
        }

        impl $t {
            fn lock_mut(&mut self) -> UniqueMemoryGuard<$actual, RwLockWriteGuard<()>> {
                unsafe { UniqueMemoryGuard::new(
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

                let half = $t {
                    start_paddr: paddr,
                    lock: RwLock::new(()),
                    deleted: false,
                };

                for entry in half.lock_mut().iter_mut() {
                    *entry = $entry::empty();
                }

                half
            }

            pub fn $map_name(&mut self, index: usize, sub: &mut $sub_half) {
                let current = self.lock_mut();
                assert!(!current[index].is_present());

                current[index] = $entry::new(sub.start_paddr(), $access);
            }
        }

    }
}

paging_half!(PTHalf, PageHalf, PT, PTEntry, PT_P | PT_RW | PT_US, map_page);
paging_half!(PDPTHalf, PDHalf, PDPT, PDPTEntry, PDPT_P | PDPT_RW | PDPT_US, map_pd);
paging_half!(PDHalf, PTHalf, PD, PDEntry, PD_P | PD_RW | PD_US, map_pt);
