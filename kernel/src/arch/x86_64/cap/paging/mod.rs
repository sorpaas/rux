mod page;
mod pml4;

pub use self::page::{PageHalf, PageFull};
pub use self::pml4::{PML4Half, PML4Full};

use common::*;
use arch::paging::{BASE_PAGE_LENGTH,
                   PT, PTEntry, PT_P, PT_RW, PT_US,
                   PD, PDEntry, PD_P, PD_RW, PD_US,
                   PDPT, PDPTEntry, PDPT_P, PDPT_RW, PDPT_US};
use util::{MemoryObject, UniqueReadGuard, UniqueWriteGuard,
           RwLock, RwLockReadGuard, RwLockWriteGuard};
use cap::{UntypedFull, CapFull, CapNearlyFull, MDB, Cap, CapReadRefObject};

pub type PDPTFull = CapFull<PDPTHalf, [MDB; 2]>;
pub type PDFull = CapFull<PDHalf, [MDB; 2]>;
pub type PTFull = CapFull<PTHalf, [MDB; 2]>;

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

        impl CapFull<$t, [MDB; 2]> {
            pub fn retype<'a>(untyped: &'a mut UntypedFull) -> CapNearlyFull<$t, [Option<&'a mut MDB>; 2]> {
                let alignment = BASE_PAGE_LENGTH;
                let (paddr, mdb) = untyped.allocate(BASE_PAGE_LENGTH, alignment);

                let mut half = $t {
                    start_paddr: paddr,
                    lock: RwLock::new(()),
                };

                for entry in half.write().iter_mut() {
                    *entry = $entry::empty();
                }

                CapNearlyFull::<$t, [Option<&mut MDB>; 2]>::new(half, [ mdb, None ])
            }

            pub fn $map_name(&mut self, index: usize, sub: &mut $sub_half) {
                assert!(!sub.mdb_mut(1).has_parent());
                {
                    let mut current = self.write();
                    assert!(!current[index].is_present());

                    current[index] = $entry::new(sub.start_paddr(), $access);
                }
                sub.mdb_mut(1).associate(self.mdb_mut(1));
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
        }

    }
}

paging_half!(PTHalf, PageFull, PT, PTEntry, PT_P | PT_RW | PT_US, map_page);
paging_half!(PDPTHalf, PDFull, PDPT, PDPTEntry, PDPT_P | PDPT_RW | PDPT_US, map_pd);
paging_half!(PDHalf, PTFull, PD, PDEntry, PD_P | PD_RW | PD_US, map_pt);
