use arch::paging::{PML4, PDPT, PD, PT,
                   PML4Entry, PDPTEntry, PDEntry, PTEntry,
                   BASE_PAGE_LENGTH};
use arch;

use common::{PAddr, VAddr};

use arch::cap::{ArchSpecificCapability};
use cap::{UntypedHalf, CPoolHalf, Capability, CapHalf};
use core::ops::{Index};

#[derive(Debug)]
pub struct PageHalf {
    start_paddr: PAddr,
    deleted: bool
}

normal_half!(PageHalf);

impl PageHalf {
    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }

    pub fn new(untyped: &mut UntypedHalf) -> PageHalf {
        let alignment = BASE_PAGE_LENGTH;
        let paddr = untyped.allocate(BASE_PAGE_LENGTH, alignment);

        unsafe {
            arch::with_slice_mut(paddr, BASE_PAGE_LENGTH, |s: &mut [u8]| {
                for u in s { *u = 0x0; }
            });
        }

        PageHalf {
            start_paddr: paddr,
            deleted: false
        }
    }

    pub fn length() -> usize {
        BASE_PAGE_LENGTH
    }
}

#[derive(Debug)]
pub struct PML4Half {
    start_paddr: PAddr,
    deleted: bool
}

normal_half!(PML4Half);

fn insert_in_none(slice: &mut [Option<Capability>], cap: Capability) {
    for space in slice.iter_mut() {
        if space.is_none() {
            *space = Some(cap);
            return;
        }
    }
    assert!(false);
}

impl PML4Half {
    pub fn map_pdpt(&mut self, index: usize, pdpt: &PDPTHalf) {
        use arch::paging::{pml4_index, PML4Entry, PML4_P, PML4_RW, PML4_US};
        use arch::{KERNEL_BASE};

        assert!(!(pml4_index(VAddr::from(KERNEL_BASE)) == index));

        unsafe {
            arch::with_object_mut(self.start_paddr, |pml4: &mut PML4| {
                pml4[index] = PML4Entry::new(pdpt.start_paddr, PML4_P | PML4_RW | PML4_US);
            });
        }
    }

    pub fn get_entry(&self, index: usize) -> PML4Entry {
        unsafe {
            arch::with_object(self.start_paddr, |pml4: &PML4| { pml4[index] })
        }
    }

    pub fn new(untyped: &mut UntypedHalf) -> PML4Half {
        let alignment = BASE_PAGE_LENGTH;
        let paddr = untyped.allocate(BASE_PAGE_LENGTH, alignment);

        unsafe {
            arch::with_object_mut(paddr, |pml4: &mut PML4| {
                use arch::paging::{pml4_index, PML4Entry, PML4_P, PML4_RW};
                use arch::init::{KERNEL_PDPT};
                use arch::{KERNEL_BASE};

                for entry in pml4.iter_mut() {
                    *entry = PML4Entry::empty();
                }
            
                pml4[pml4_index(VAddr::from(KERNEL_BASE))] =
                    PML4Entry::new(KERNEL_PDPT.paddr(), PML4_P | PML4_RW);
            });
        }
        
        PML4Half {
            start_paddr: paddr,
            deleted: false
        }
    }

    pub fn switch_to(&self) {
        use arch::paging;

        unsafe { paging::switch_to(self.start_paddr); }
    }

    pub fn length() -> usize {
        BASE_PAGE_LENGTH
    }
    
    pub fn map(&mut self, vaddr: VAddr, page: &PageHalf,
               untyped: &mut UntypedHalf, cpool: &mut CPoolHalf) {
        use arch::paging::{pml4_index, pdpt_index, pd_index, pt_index,
                           PML4Entry, PDPTEntry, PDEntry, PTEntry};

        cpool.with_cpool_mut(|cpool| {
            let mut slice = cpool.slice_mut();
            
            let pdpt_cap: &mut Capability = {
                let index = pml4_index(vaddr);
                
                if !self.get_entry(index).is_present() {
                    let pdpt_half = PDPTHalf::new(untyped);
                    self.map_pdpt(index, &pdpt_half);

                    insert_in_none(slice, Capability::ArchSpecific(ArchSpecificCapability::PDPT(pdpt_half)));
                }

                let position = slice.iter_mut().position(|cap: &mut Option<Capability>| {
                    match cap {
                        &mut Some(Capability::ArchSpecific(ArchSpecificCapability::PDPT(ref mut pdpt_half))) =>
                            pdpt_half.start_paddr == self.get_entry(index).get_address(),
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
                
                if !pdpt_half.get_entry(index).is_present() {
                    let pd_half = PDHalf::new(untyped);
                    pdpt_half.map_pd(index, &pd_half);

                    insert_in_none(slice, Capability::ArchSpecific(ArchSpecificCapability::PD(pd_half)));
                }

                let position = slice.iter_mut().position(|cap: &mut Option<Capability>| {
                    match cap {
                        &mut Some(Capability::ArchSpecific(ArchSpecificCapability::PD(ref mut pd_half))) =>
                            pd_half.start_paddr == pdpt_half.get_entry(index).get_address(),
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
                
                if !pd_half.get_entry(index).is_present() {
                    let pt_half = PTHalf::new(untyped);
                    pd_half.map_pt(index, &pt_half);

                    insert_in_none(slice, Capability::ArchSpecific(ArchSpecificCapability::PT(pt_half)));
                }

                let position = slice.iter_mut().position(|cap: &mut Option<Capability>| {
                    match cap {
                        &mut Some(Capability::ArchSpecific(ArchSpecificCapability::PT(ref mut pt_half))) =>
                            pt_half.start_paddr == pd_half.get_entry(index).get_address(),
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
        });
    }
}

#[derive(Debug)]
pub struct PDPTHalf {
    start_paddr: PAddr,
    deleted: bool
}

normal_half!(PDPTHalf);

impl PDPTHalf {
    pub fn new(untyped: &mut UntypedHalf) -> PDPTHalf {
        let alignment = BASE_PAGE_LENGTH;
        let paddr = untyped.allocate(BASE_PAGE_LENGTH, alignment);

        unsafe {
            arch::with_object_mut(paddr, |pdpt: &mut PDPT| {
                use arch::paging::{PDPTEntry};

                for entry in pdpt.iter_mut() {
                    *entry = PDPTEntry::empty();
                }
            });
        }

        PDPTHalf {
            start_paddr: paddr,
            deleted: false
        }
    }

    pub fn map_pd(&mut self, index: usize, pd: &PDHalf) {
        use arch::paging::{PDPTEntry, PDPT_P, PDPT_RW, PDPT_US};

        unsafe {
            arch::with_object_mut(self.start_paddr, |pdpt: &mut PDPT| {
                pdpt[index] = PDPTEntry::new(pd.start_paddr, PDPT_P | PDPT_RW | PDPT_US);
            });
        }
    }

    pub fn get_entry(&self, index: usize) -> PDPTEntry {
        unsafe {
            arch::with_object(self.start_paddr, |pdpt: &PDPT| { pdpt[index] })
        }
    }
}

#[derive(Debug)]
pub struct PDHalf {
    start_paddr: PAddr,
    deleted: bool
}

normal_half!(PDHalf);

impl PDHalf {
    pub fn new(untyped: &mut UntypedHalf) -> PDHalf {
        let alignment = BASE_PAGE_LENGTH;
        let paddr = untyped.allocate(BASE_PAGE_LENGTH, alignment);

        unsafe {
            arch::with_object_mut(paddr, |pdpt: &mut PD| {
                use arch::paging::{PDEntry};
                
                for entry in pdpt.iter_mut() {
                    *entry = PDEntry::empty();
                }
            });
        }

        PDHalf {
            start_paddr: paddr,
            deleted: false
        }
    }

    pub fn map_pt(&mut self, index: usize, pt: &PTHalf) {
        use arch::paging::{PDEntry, PD_P, PD_RW, PD_US};

        unsafe {
            arch::with_object_mut(self.start_paddr, |pd: &mut PD| {
                pd[index] = PDEntry::new(pt.start_paddr, PD_P | PD_RW | PD_US);
            });
        }
    }

    pub fn get_entry(&self, index: usize) -> PDEntry {
        unsafe {
            arch::with_object(self.start_paddr, |pd: &PD| { pd[index] })
        }
    }
}

#[derive(Debug)]
pub struct PTHalf {
    start_paddr: PAddr,
    deleted: bool
}

normal_half!(PTHalf);

impl PTHalf {
    pub fn new(untyped: &mut UntypedHalf) -> PTHalf {
        let alignment = BASE_PAGE_LENGTH;
        let paddr = untyped.allocate(BASE_PAGE_LENGTH, alignment);

        unsafe {
            arch::with_object_mut(paddr, |pt: &mut PT| {
                use arch::paging::{PTEntry};
                
                for entry in pt.iter_mut() {
                    *entry = PTEntry::empty();
                }
            });
        }

        PTHalf {
            start_paddr: paddr,
            deleted: false
        }
    }

    pub fn map_page(&mut self, index: usize, page: &PageHalf) {
        use arch::paging::{PTEntry, PT_P, PT_RW, PT_US};

        unsafe {
            arch::with_object_mut(self.start_paddr, |pt: &mut PT| {
                pt[index] = PTEntry::new(page.start_paddr, PT_P | PT_RW | PT_US);
            });
        }
    }

    pub fn get_entry(&self, index: usize) -> PTEntry {
        unsafe {
            arch::with_object(self.start_paddr, |pt: &PT| { pt[index] })
        }
    }
}
