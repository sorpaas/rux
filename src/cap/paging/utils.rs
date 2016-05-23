use common::*;

use core::ptr::Unique;
use super::table::{PageTable, PageTableLevel4};

pub unsafe fn map_in_active(frame_start_addr: PhysicalAddress, dest_addr: usize, flags: EntryFlags) {
    let mut unique = unsafe { Unique::new(0xffffffff_fffff000 as *mut _) };
    let page = Page::new(dest_addr);
    let mut p4: &mut PageTable<PageTableLevel4> = unsafe { unique.get_mut() };
    let mut p3 = p4.next_table_mut_in_active(page.p4_index()).unwrap();
    let mut p2 = p3.next_table_mut_in_active(page.p3_index()).unwrap();
    let mut p1 = p2.next_table_mut_in_active(page.p2_index()).unwrap();

    assert!(p1[page.p1_index()].is_unused());
    p1[page.p1_index()].set_address(frame_start_addr, flags | PRESENT);
}

pub unsafe fn unmap_in_active(dest_addr: usize) {
    let mut unique = unsafe { Unique::new(0xffffffff_fffff000 as *mut _) };
    let page = Page::new(dest_addr);
    let mut p4: &mut PageTable<PageTableLevel4> = unsafe { unique.get_mut() };
    let mut p3 = p4.next_table_mut_in_active(page.p4_index()).unwrap();
    let mut p2 = p3.next_table_mut_in_active(page.p3_index()).unwrap();
    let mut p1 = p2.next_table_mut_in_active(page.p2_index()).unwrap();

    assert!(p1[page.p1_index()].is_unused());
    p1[page.p1_index()].set_unused();
}

#[derive(Clone, Copy)]
pub struct Page {
    number: usize,
}

impl Page {
    pub fn new(address: usize) -> Page {
        assert!(address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000, "invalid address: 0x{:x}", address);
        Page { number: address / PAGE_SIZE }
    }

    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    pub fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    pub fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    pub fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    pub fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
}
