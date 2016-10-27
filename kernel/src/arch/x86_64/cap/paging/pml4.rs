use common::*;
use arch::paging::{PML4, PML4Entry, PML4_P, PML4_RW, PML4_US, BASE_PAGE_LENGTH, pml4_index};
use util::{MemoryObject, UniqueReadGuard, UniqueWriteGuard,
           RwLock, RwLockReadGuard, RwLockWriteGuard,
           SharedReadGuard, SharedWriteGuard};
use cap::{Cap, UntypedFull, CapFull, MDB, CapNearlyFull, CapReadRefObject, CapWriteRefObject,
          ArchCap, CPoolHalf, CPoolFull};
use core::ops::{Deref, DerefMut};

use super::{PageHalf, PageFull, PDPTHalf, PDPTFull, PDHalf, PDFull, PTHalf, PTFull};

/// Non-clonable, lock in CapHalf

pub type PML4NearlyFull<'a> = CapNearlyFull<PML4Half, [Option<&'a mut MDB>; 2]>;
pub type PML4Full = CapFull<PML4Half, [MDB; 2]>;

impl PML4Full {
    pub fn retype<'a>(untyped: &'a mut UntypedFull) -> PML4NearlyFull<'a> {
        use arch::init::{KERNEL_PDPT};
        use arch::{KERNEL_BASE};

        let alignment = BASE_PAGE_LENGTH;
        let (paddr, mdb) = untyped.allocate(BASE_PAGE_LENGTH, alignment);

        let mut half = PML4Half {
            start_paddr: paddr,
            lock: RwLock::new(()),
        };

        {
            let mut pml4 = half.write();

            for entry in pml4.iter_mut() {
                *entry = PML4Entry::empty();
            }
            pml4[pml4_index(VAddr::from(KERNEL_BASE))] =
                PML4Entry::new(KERNEL_PDPT.paddr(), PML4_P | PML4_RW);
        }

        PML4NearlyFull::new(half, [ mdb, None ])
    }


    pub fn map_pdpt(&mut self, index: usize, pdpt: &mut PDPTFull) {
        use arch::{KERNEL_BASE};

        assert!(!pdpt.mdb_mut(1).has_parent());
        {
            let mut pml4 = self.write();

            assert!(!(pml4_index(VAddr::from(KERNEL_BASE)) == index));
            assert!(!pml4[index].is_present());

            pml4[index] = PML4Entry::new(pdpt.start_paddr(), PML4_P | PML4_RW | PML4_US);
        }
        pdpt.mdb_mut(1).associate(self.mdb_mut(1));
    }

    pub fn map(&mut self, vaddr: VAddr, page: &mut PageFull,
               untyped: &mut UntypedFull, cpool: &mut CPoolFull) {
        use arch::paging::{pml4_index, pdpt_index, pd_index, pt_index,
                           PML4Entry, PDPTEntry, PDEntry, PTEntry};
        let mut pdpt_cap = {
            let index = pml4_index(vaddr);

            if !{ self.read()[index] }.is_present() {
                let mut pdpt_nearly = PDPTFull::retype(untyped);
                let inserted_index = cpool.insert(pdpt_nearly);
                let mut pdpt = cpool.write(inserted_index);
                self.map_pdpt(index, match pdpt.as_mut().unwrap() {
                    &mut Cap::Arch(ArchCap::PDPT(ref mut pdpt)) =>
                        pdpt,
                    _ => panic!()
                });
            }

            let position = (0..cpool.size()).position(|i| {
                let cap = cpool.read(i);
                match cap.deref() {
                    &Some(Cap::Arch(ArchCap::PDPT(ref pdpt))) =>
                        pdpt.start_paddr() == { self.read()[index] }.get_address(),
                    _ => false
                }
            }).unwrap();

            Some(cpool.write(position))
        }.unwrap();

        let pdpt = {
            match pdpt_cap.as_mut().unwrap() {
                &mut Cap::Arch(ArchCap::PDPT(ref mut pdpt)) =>
                    pdpt,
                _ => panic!()
            }
        };

        log!("pdpt: {:?}", pdpt);

        let mut pd_cap = {
            let index = pdpt_index(vaddr);

            if !{ self.read()[index] }.is_present() {
                let mut pd_nearly = PDFull::retype(untyped);
                let inserted_index = cpool.insert(pd_nearly);
                let mut pd = cpool.write(inserted_index);
                pdpt.map_pd(index, match pd.as_mut().unwrap() {
                    &mut Cap::Arch(ArchCap::PD(ref mut pd)) =>
                        pd,
                    _ => panic!()
                });
            }

            let position = (0..cpool.size()).position(|i| {
                let cap = cpool.read(i);
                match cap.deref() {
                    &Some(Cap::Arch(ArchCap::PD(ref pd))) =>
                        pd.start_paddr() == { self.read()[index] }.get_address(),
                    _ => false
                }
            }).unwrap();

            Some(cpool.write(position))
        }.unwrap();

        let pd = {
            match pd_cap.as_mut().unwrap() {
                &mut Cap::Arch(ArchCap::PD(ref mut pd)) =>
                    pd,
                _ => panic!()
            }
        };

        log!("pd: {:?}", pd);

        let mut pt_cap = {
            let index = pd_index(vaddr);

            if !{ self.read()[index] }.is_present() {
                let mut pt_nearly = PTFull::retype(untyped);
                let inserted_index = cpool.insert(pt_nearly);
                let mut pt = cpool.write(inserted_index);
                pd.map_pt(index, match pt.as_mut().unwrap() {
                    &mut Cap::Arch(ArchCap::PT(ref mut pt)) =>
                        pt,
                    _ => panic!()
                });
            }

            let position = (0..cpool.size()).position(|i| {
                let cap = cpool.read(i);
                match cap.deref() {
                    &Some(Cap::Arch(ArchCap::PT(ref pt))) =>
                        pt.start_paddr() == { self.read()[index] }.get_address(),
                    _ => false
                }
            }).unwrap();

            Some(cpool.write(position))
        }.unwrap();

        let pt = {
            match pt_cap.as_mut().unwrap() {
                &mut Cap::Arch(ArchCap::PT(ref mut pt)) =>
                    pt,
                _ => panic!()
            }
        };

        log!("pt: {:?}", pt);

        pt.map_page(pt_index(vaddr), page);
    }
}

#[derive(Debug)]
pub struct PML4Half {
    start_paddr: PAddr,
    lock: RwLock<()>,
}

impl<'a> CapReadRefObject<'a, PML4, UniqueReadGuard<'a, PML4>> for PML4Half {
    fn read(&'a self) -> UniqueReadGuard<'a, PML4> {
        unsafe { UniqueReadGuard::new(
            MemoryObject::<PML4>::new(self.start_paddr),
            self.lock.read()
        ) }
    }
}

impl PML4Half {
    fn write(&mut self) -> UniqueWriteGuard<PML4> {
        unsafe { UniqueWriteGuard::new(
            MemoryObject::<PML4>::new(self.start_paddr),
            self.lock.write()
        ) }
    }

    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }

    pub fn length() -> usize {
        BASE_PAGE_LENGTH
    }

    pub fn switch_to(&self) {
        use arch::paging;

        unsafe { paging::switch_to(self.start_paddr); }
    }
}
