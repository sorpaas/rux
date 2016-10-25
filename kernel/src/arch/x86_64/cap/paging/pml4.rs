use common::*;
use arch::paging::{PML4, PML4Entry, PML4_P, PML4_RW, PML4_US, BASE_PAGE_LENGTH, pml4_index};
use util::{MemoryObject, UniqueReadGuard, UniqueWriteGuard,
           RwLock, RwLockReadGuard, RwLockWriteGuard};
use cap::{UntypedHalf, Capability, CapReadObject, CapWriteObject,
          CapHalf, ArchSpecificCapability, CPoolHalf};

use super::{PageHalf, PDPTHalf, PDHalf, PTHalf};

/// Non-clonable, lock in CapHalf

#[derive(Debug)]
pub struct PML4Half {
    start_paddr: PAddr,
    lock: RwLock<()>,
    deleted: bool
}

normal_half!(PML4Half);

impl<'a> CapReadObject<'a, PML4, UniqueReadGuard<'a, PML4>> for PML4Half {
    fn read(&self) -> UniqueReadGuard<PML4> {
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

    pub fn new(untyped: &mut UntypedHalf) -> PML4Half {
        use arch::init::{KERNEL_PDPT};
        use arch::{KERNEL_BASE};

        let alignment = BASE_PAGE_LENGTH;
        let paddr = untyped.allocate(BASE_PAGE_LENGTH, alignment);

        let mut half = PML4Half {
            start_paddr: paddr,
            lock: RwLock::new(()),
            deleted: false
        };

        {
            let mut pml4 = half.write();

            for entry in pml4.iter_mut() {
                *entry = PML4Entry::empty();
            }
            pml4[pml4_index(VAddr::from(KERNEL_BASE))] =
                PML4Entry::new(KERNEL_PDPT.paddr(), PML4_P | PML4_RW);
        }

        half
    }

    pub fn map_pdpt(&mut self, index: usize, pdpt: &PDPTHalf) {
        use arch::{KERNEL_BASE};

        let mut pml4 = self.write();

        assert!(!(pml4_index(VAddr::from(KERNEL_BASE)) == index));
        assert!(!pml4[index].is_present());

        pml4[index] = PML4Entry::new(pdpt.start_paddr(), PML4_P | PML4_RW | PML4_US);
    }

    fn insert_in_none(slice: &mut [Option<Capability>], cap: Capability) {
        for space in slice.iter_mut() {
            if space.is_none() {
                *space = Some(cap);
                return;
            }
        }
        assert!(false);
    }

    pub fn switch_to(&self) {
        use arch::paging;

        unsafe { paging::switch_to(self.start_paddr); }
    }

    pub fn map(&mut self, vaddr: VAddr, page: &mut PageHalf,
               untyped: &mut UntypedHalf, cpool_half: &mut CPoolHalf) {
        use arch::paging::{pml4_index, pdpt_index, pd_index, pt_index,
                           PML4Entry, PDPTEntry, PDEntry, PTEntry};

        let mut cpool = cpool_half.write();
        let mut slice = cpool.slice_mut();

        let pdpt_cap: &mut Capability = {
            let index = pml4_index(vaddr);

            if !{ self.read()[index] }.is_present() {
                let mut pdpt_half = PDPTHalf::new(untyped);
                self.map_pdpt(index, &mut pdpt_half);

                Self::insert_in_none(slice,
                                     Capability::ArchSpecific(ArchSpecificCapability::PDPT(pdpt_half)));
            }

            let position = slice.iter_mut().position(|cap: &mut Option<Capability>| {
                match cap {
                    &mut Some(Capability::ArchSpecific(ArchSpecificCapability::PDPT(ref mut pdpt_half))) =>
                        pdpt_half.start_paddr == { self.read()[index] }.get_address(),
                    _ => false,
                }
            }).unwrap();

            unsafe { &mut (*(&slice[position] as *const Option<Capability> as u64 as *mut Option<Capability>)) }
        }.as_mut().unwrap();

        let pdpt_half: &mut PDPTHalf = {
            match pdpt_cap {
                &mut Capability::ArchSpecific(ArchSpecificCapability::PDPT(ref mut pdpt_half)) => pdpt_half,
                _ => panic!(),
            }
        };

        log!("pdpt_half: {:?}", pdpt_half);

        let pd_cap: &mut Capability = {
            let index = pdpt_index(vaddr);

            if !{ pdpt_half.read()[index] }.is_present() {
                let mut pd_half = PDHalf::new(untyped);
                pdpt_half.map_pd(index, &mut pd_half);

                Self::insert_in_none(slice, Capability::ArchSpecific(ArchSpecificCapability::PD(pd_half)));
            }

            let position = slice.iter_mut().position(|cap: &mut Option<Capability>| {
                match cap {
                    &mut Some(Capability::ArchSpecific(ArchSpecificCapability::PD(ref mut pd_half))) =>
                            pd_half.start_paddr == { pdpt_half.read()[index] }.get_address(),
                    _ => false,
                }
            }).unwrap();

            unsafe { &mut (*(&slice[position] as *const Option<Capability> as u64 as *mut Option<Capability>)) }
        }.as_mut().unwrap();

        let pd_half: &mut PDHalf = {
            match pd_cap {
                &mut Capability::ArchSpecific(ArchSpecificCapability::PD(ref mut pd_half)) => pd_half,
                _ => panic!(),
            }
        };

        log!("pd_half: {:?}", pd_half);

        let pt_cap: &mut Capability = {
            let index = pd_index(vaddr);

            if !{ pd_half.read()[index] }.is_present() {
                let mut pt_half = PTHalf::new(untyped);
                pd_half.map_pt(index, &mut pt_half);

                Self::insert_in_none(slice, Capability::ArchSpecific(ArchSpecificCapability::PT(pt_half)));
            }

            let position = slice.iter_mut().position(|cap: &mut Option<Capability>| {
                match cap {
                    &mut Some(Capability::ArchSpecific(ArchSpecificCapability::PT(ref mut pt_half))) =>
                        pt_half.start_paddr == { pd_half.read()[index] }.get_address(),
                    _ => false,
                }
            }).unwrap();

            unsafe { &mut (*(&slice[position] as *const Option<Capability> as u64 as *mut Option<Capability>)) }
        }.as_mut().unwrap();

        let pt_half: &mut PTHalf = {
            match pt_cap {
                &mut Capability::ArchSpecific(ArchSpecificCapability::PT(ref mut pt_half)) => pt_half,
                _ => panic!(),
            }
        };

        log!("pt_half: {:?}", pt_half);

        pt_half.map_page(pt_index(vaddr), page);
    }
}
