use common::*;
use arch::{KERNEL_BASE};
use arch::init::{KERNEL_PDPT};
use arch::paging::{BASE_PAGE_LENGTH, PML4, PML4Entry, pml4_index};
use util::{MemoryObject, UniqueReadGuard, UniqueWriteGuard, RwLock};
use super::{PML4Descriptor, PML4Cap, PDPTCap, PDCap, PTCap, PageCap};
use cap::{self, UntypedDescriptor, CPoolDescriptor, SetDefault};
use core::any::Any;

impl PML4Cap {
    pub fn retype_from(untyped: &mut UntypedDescriptor) -> Self {
        let mut arc: Option<Self> = None;

        let start_paddr = unsafe { untyped.allocate(BASE_PAGE_LENGTH, BASE_PAGE_LENGTH) };

        unsafe {
            use arch::paging::{PML4_P, PML4_RW};

            untyped.derive(Self::inner_length(), Self::inner_alignment(), |paddr, next_child| {
                let mut desc = PML4Descriptor {
                    start_paddr: start_paddr,
                    next: next_child,
                };

                for item in desc.write().iter_mut() {
                    *item = PML4Entry::empty();
                }

                desc.write()[pml4_index(VAddr::from(KERNEL_BASE))] =
                    PML4Entry::new(KERNEL_PDPT.paddr(), PML4_P | PML4_RW);

                arc = Some(
                    Self::new(paddr, RwLock::new(desc))
                );

                arc.clone().unwrap().into()
            });
        }

        arc.unwrap()
    }

    pub fn map_pdpt(&mut self, index: usize, sub: &PDPTCap) {
        use arch::paging::{pml4_index, PML4_P, PML4_RW, PML4_US};

        let mut current_desc = self.write();
        let mut current = current_desc.write();
        let sub_desc = sub.read();
        assert!(!(pml4_index(VAddr::from(KERNEL_BASE)) == index));
        assert!(!current[index].is_present());

        sub_desc.mapped_weak_pool.read().downgrade_at(self, 0);
        current[index] = PML4Entry::new(sub_desc.start_paddr(), PML4_P | PML4_RW | PML4_US);
    }

    pub fn map<T: SetDefault + Any>(&mut self, vaddr: VAddr, page: &PageCap<T>,
                                    untyped: &mut UntypedDescriptor, cpool: &mut CPoolDescriptor) {
        use arch::paging::{pml4_index, pdpt_index, pd_index, pt_index};

        log!("PML4 mapping: 0x{:x}", vaddr);

        let mut pdpt_cap: PDPTCap = {
            let index = pml4_index(vaddr);

            if !{ self.read().read()[index] }.is_present() {
                let pdpt_cap = PDPTCap::retype_from(untyped);
                self.map_pdpt(index, &pdpt_cap);
                cpool.downgrade_free(&pdpt_cap);
            }

            let position = (0..cpool.size()).position(|i| {
                let any = cpool.upgrade_any(i);
                if let Some(any) = any {
                    if any.is::<PDPTCap>() {
                        let cap: PDPTCap = any.into();
                        let cap_desc = cap.read();
                        cap_desc.start_paddr() == { self.read().read()[index] }.get_address()
                    } else {
                        cap::drop_any(any);
                        false
                    }
                } else {
                    false
                }
            }).unwrap();

            cpool.upgrade(position)
        }.unwrap();

        let mut pd_cap: PDCap = {
            let index = pdpt_index(vaddr);

            if !{ pdpt_cap.read().read()[index] }.is_present() {
                let pd_cap = PDCap::retype_from(untyped);
                pdpt_cap.map_pd(index, &pd_cap);
                cpool.downgrade_free(&pd_cap);
            }

            let position = (0..cpool.size()).position(|i| {
                let any = cpool.upgrade_any(i);
                if let Some(any) = any {
                    if any.is::<PDCap>() {
                        let cap: PDCap = any.into();
                        let cap_desc = cap.read();
                        cap_desc.start_paddr() == { pdpt_cap.read().read()[index] }.get_address()
                    } else {
                        cap::drop_any(any);
                        false
                    }
                } else {
                    false
                }
            }).unwrap();

            cpool.upgrade(position)
        }.unwrap();

        let mut pt_cap: PTCap = {
            let index = pd_index(vaddr);

            if !{ pd_cap.read().read()[index] }.is_present() {
                let pt_cap = PTCap::retype_from(untyped);
                pd_cap.map_pt(index, &pt_cap);
                cpool.downgrade_free(&pt_cap);
            }

            let position = (0..cpool.size()).position(|i| {
                let any = cpool.upgrade_any(i);
                if let Some(any) = any {
                    if any.is::<PTCap>() {
                        let cap: PTCap = any.into();
                        let cap_desc = cap.read();
                        cap_desc.start_paddr() == { pd_cap.read().read()[index] }.get_address()
                    } else {
                        cap::drop_any(any);
                        false
                    }
                } else {
                    false
                }
            }).unwrap();

            cpool.upgrade(position)
        }.unwrap();

        pt_cap.map_page(pt_index(vaddr), page);
    }
}

impl PML4Descriptor {
    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }

    pub fn length(&self) -> usize {
        BASE_PAGE_LENGTH
    }

    fn page_object(&self) -> MemoryObject<PML4> {
        unsafe { MemoryObject::new(self.start_paddr) }
    }

    pub fn read(&self) -> UniqueReadGuard<PML4> {
        unsafe { UniqueReadGuard::new(self.page_object()) }
    }

    fn write(&mut self) -> UniqueWriteGuard<PML4> {
        unsafe { UniqueWriteGuard::new(self.page_object()) }
    }

    pub fn switch_to(&mut self) {
        use arch::paging;

        unsafe { paging::switch_to(self.start_paddr); }
    }
}
