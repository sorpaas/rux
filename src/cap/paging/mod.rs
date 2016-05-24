mod table;
mod entry;
mod mapper;
mod utils;

pub use self::mapper::{Mapper, active_mapper};

use common::*;
use core::marker::PhantomData;
use core::ptr::Unique;

use super::{UntypedCapability, MemoryBlock,
            FrameCapability};
use self::mapper::{switch_mapper};
use self::table::{PageTable, PageTableLevel4, PageTableLevel3, PageTableLevel2, PageTableLevel1};
use self::utils::{Page};

use x86::controlregs;

pub struct PageTableCapability {
    p4_block: MemoryBlock,
    active_mappable_virtual_start_addr: usize,
    active_mappable_count: usize,
    useless: bool,
}

impl PageTableCapability {
    pub fn p4_block(&self) -> &MemoryBlock {
        &self.p4_block
    }

    fn map_addr(&self, frame_addr: usize, dest_addr: usize, flags: EntryFlags) {
        let page = Page::new(dest_addr);

        unsafe {
            active_mapper().borrow_mut_map(self.p4_block().start_addr(), 1, |p4 : &mut PageTable<PageTableLevel4>, mapper| {
                mapper.borrow_mut_map(p4[page.p4_index()].physical_address().unwrap(), 1, |p3 : &mut PageTable<PageTableLevel3>, mapper| {
                    mapper.borrow_mut_map(p3[page.p3_index()].physical_address().unwrap(), 1, |p2 : &mut PageTable<PageTableLevel2>, mapper| {
                        mapper.borrow_mut_map(p2[page.p2_index()].physical_address().unwrap(), 1, |p1 : &mut PageTable<PageTableLevel1>, mapper| {
                            assert!(p1[page.p1_index()].is_unused());
                            p1[page.p1_index()].set_address(frame_addr, flags | PRESENT);
                        })
                    })
                })
            })
        }
    }

    fn unmap_addr(&self, dest_addr: usize) -> PhysicalAddress {
        let page = Page::new(dest_addr);

        unsafe {
            let mut frame_addr = None;

            active_mapper().borrow_mut_map(self.p4_block().start_addr(), 1, |p4 : &mut PageTable<PageTableLevel4>, mapper| {
                mapper.borrow_mut_map(p4[page.p4_index()].physical_address().unwrap(), 1, |p3 : &mut PageTable<PageTableLevel3>, mapper| {
                    mapper.borrow_mut_map(p3[page.p3_index()].physical_address().unwrap(), 1, |p2 : &mut PageTable<PageTableLevel2>, mapper| {
                        mapper.borrow_mut_map(p2[page.p2_index()].physical_address().unwrap(), 1, |p1 : &mut PageTable<PageTableLevel1>, mapper| {
                            assert!(!p1[page.p1_index()].is_unused());
                            frame_addr = p1[page.p1_index()].physical_address();
                            p1[page.p1_index()].set_unused();
                        })
                    })
                })
            });

            frame_addr.unwrap()
        }
    }

    pub unsafe fn create_tables_and_map_vga_buffer(&mut self, untyped: &mut UntypedCapability) {
        self.create_tables(0xb8000, 1, untyped);
        self.map_addr(0xb8000, 0xb8000, WRITABLE);
    }

    pub fn create_tables(&mut self, dest_addr: usize, count: usize, untyped: &mut UntypedCapability) {
        for i in 0..count {
            let page = Page::new(dest_addr + i * PAGE_SIZE);

            unsafe {
                active_mapper().borrow_mut_map(self.p4_block().start_addr(), 1, |p4 : &mut PageTable<PageTableLevel4>, mapper| {
                    p4.next_table_create(page.p4_index(), untyped, mapper, |p3, untyped, mapper| {
                        p3.next_table_create(page.p3_index(), untyped, mapper, |p2, untyped, mapper| {
                            p2.next_table_create(page.p2_index(), untyped, mapper, |p1, untyped, mapper| { })
                        })
                    })
                });
            }
        }
    }

    pub fn map(&mut self, mut frame: FrameCapability, dest_addr: usize) {
        for i in 0..frame.count() {
            let frame_addr = frame.block().start_addr() + i * PAGE_SIZE;
            let flags = frame.flags();
            self.map_addr(frame_addr, dest_addr + i * PAGE_SIZE, flags);
        }

        unsafe { frame.mark_useless(); }
    }

    pub fn unmap(&mut self, dest_addr: usize) -> FrameCapability {
        unimplemented!()
    }

    pub fn identity_map(&mut self, frame: FrameCapability) {
        let dest_addr = frame.block().start_addr();
        self.map(frame, dest_addr)
    }

    pub fn create_tables_and_map(&mut self, frame: FrameCapability, dest_addr: usize, untyped: &mut UntypedCapability) {
        let frame_start_addr = frame.block().start_addr();
        let frame_count = frame.count();

        self.create_tables(dest_addr, frame_count, untyped);
        self.map(frame, dest_addr);
    }

    pub fn create_tables_and_identity_map(&mut self, frame: FrameCapability, untyped: &mut UntypedCapability) {
        let dest_addr = frame.block().start_addr();

        self.create_tables_and_map(frame, dest_addr, untyped);
    }

    pub unsafe fn switch_on(&self) {
        switch_mapper(self.active_mappable_virtual_start_addr, self.active_mappable_count);
        controlregs::cr3_write(self.p4_block().start_addr() as u64);
    }

    pub unsafe fn mark_useless(&mut self) {
        self.useless = true;
    }

    pub fn from_untyped(untyped: &mut UntypedCapability, active_mappable_virtual_start_addr: usize, active_mappable_count: usize) -> PageTableCapability {
        let p4_block = untyped.retype(PAGE_SIZE, PAGE_SIZE);

        unsafe {
            active_mapper().borrow_mut_map(p4_block.start_addr(), 1, |p4 : &mut PageTable<PageTableLevel4>, mapper| {
                p4.zero();
                p4[511].set_address(p4_block.start_addr(), PRESENT | WRITABLE);
            });
        }

        let mut target = PageTableCapability {
            p4_block: p4_block,
            active_mappable_virtual_start_addr: active_mappable_virtual_start_addr,
            active_mappable_count: active_mappable_count,
            useless: false,
        };

        target.create_tables(active_mappable_virtual_start_addr, active_mappable_count, untyped);

        target
    }
}

impl Drop for PageTableCapability{
    fn drop(&mut self) {
        unsafe { self.p4_block.mark_useless(); }
        if self.useless { return; }

        unimplemented!();
    }
}
