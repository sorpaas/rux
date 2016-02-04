use core::ptr::Unique;
use core::marker::PhantomData;

use common::*;
use cap::PageBlockCapability;
use cap::UntypedCapability;
use super::VirtualAddress;
use super::table::{PageTable, PageTableLevel4};
use super::entry::EntryFlags;

#[derive(Debug, Clone, Copy)]
pub struct Page {
    number: usize,
}

impl Page {
    pub fn new<U>(virt: &VirtualAddress<U>) -> Page {
        let address = virt.addr;
        assert!(address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000, "invalid address: 0x{:x}", address);
        Page { number: address / PAGE_SIZE }
    }

    fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
}

pub trait Mapper {
    fn p4(&self) -> &PageTable<PageTableLevel4>;
    fn p4_mut(&mut self) -> &mut PageTable<PageTableLevel4>;

    fn translate<U>(&self, virt: &VirtualAddress<U>) -> Option<PhysicalAddress> {
        let page = Page::new(virt);

        self.p4().next_table(page.p4_index())
            .and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].physical_address())
    }

    fn map_to<T, U>(&mut self,
                    virt: &VirtualAddress<U>,
                    block: &T,
                    flags: EntryFlags,
                    untyped: UntypedCapability)
        -> Option<UntypedCapability>
        where T: PageBlockCapability<U> {
        use super::entry::PRESENT;

        let page = Page::new(virt);

        let mut p4 = self.p4_mut();
        let (mut p3, untyped) = p4.next_table_create(page.p4_index(), untyped);
        let untyped = untyped.expect("Out of memory.");
        let (mut p2, untyped) = p3.next_table_create(page.p3_index(), untyped);
        let untyped = untyped.expect("Out of memory.");
        let (mut p1, untyped) = p2.next_table_create(page.p2_index(), untyped);

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set_address(block.page_start_addr(), flags | PRESENT);

        untyped
    }
}

pub struct ActiveMapper(Unique<PageTable<PageTableLevel4>>);

impl Mapper for ActiveMapper {
    fn p4(&self) -> &PageTable<PageTableLevel4> {
        unsafe { self.0.get() }
    }

    fn p4_mut(&mut self) -> &mut PageTable<PageTableLevel4> {
        unsafe { self.0.get_mut() }
    }

}

impl ActiveMapper {
    pub unsafe fn new() -> ActiveMapper {
        ActiveMapper(Unique::new(0xffffffff_fffff000 as *mut _))
    }

    pub unsafe fn with<F, R>(&mut self,
                             table_start_addr: PhysicalAddress,
                             f: F) -> R
        where F: FnOnce(&mut InsideMapper) -> R {

        use x86::{controlregs, tlb};
        let flush_tlb = || unsafe { tlb::flush_all() };

        {
            use super::entry::{PRESENT, WRITABLE};

            let backup = unsafe { controlregs::cr3() } as usize;

            // overwrite recursive mapping
            self.p4_mut()[511].set_address(table_start_addr, PRESENT | WRITABLE);
            flush_tlb();

            let mut mapper = InsideMapper(Unique::new(0xffffffff_fffff000 as *mut _));
            // execute f in the new context
            let r = f(&mut mapper);

            // restore recursive mapping to original p4 table
            self.p4_mut()[511].set_address(backup, PRESENT | WRITABLE);
            flush_tlb();

            r
        }
    }
}

pub struct InsideMapper(Unique<PageTable<PageTableLevel4>>);

impl Mapper for InsideMapper {
    fn p4(&self) -> &PageTable<PageTableLevel4> {
        unsafe { self.0.get() }
    }

    fn p4_mut(&mut self) -> &mut PageTable<PageTableLevel4> {
        unsafe { self.0.get_mut() }
    }
}
