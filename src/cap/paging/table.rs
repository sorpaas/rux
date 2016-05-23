use common::*;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use super::mapper::{Mapper};
use super::entry::*;
use super::super::{MemoryBlock, UntypedCapability};

pub trait PageTableLevel { }

pub enum PageTableLevel4 { }
pub enum PageTableLevel3 { }
pub enum PageTableLevel2 { }
pub enum PageTableLevel1 { }

impl PageTableLevel for PageTableLevel4 { }
impl PageTableLevel for PageTableLevel3 { }
impl PageTableLevel for PageTableLevel2 { }
impl PageTableLevel for PageTableLevel1 { }

trait PageTableHierarchicalLevel: PageTableLevel {
    type NextLevel: PageTableLevel;
}
impl PageTableHierarchicalLevel for PageTableLevel4 {
    type NextLevel = PageTableLevel3;
}
impl PageTableHierarchicalLevel for PageTableLevel3 {
    type NextLevel = PageTableLevel2;
}
impl PageTableHierarchicalLevel for PageTableLevel2 {
    type NextLevel = PageTableLevel1;
}

pub struct PageTable<L: PageTableLevel> {
    entries: [PageTableEntry; PAGE_TABLE_ENTRY_COUNT],
    level: PhantomData<L>,
}

impl<L> Index<usize> for PageTable<L> where L: PageTableLevel {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &PageTableEntry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for PageTable<L> where L: PageTableLevel {
    fn index_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
}

impl<L> PageTable<L> where L: PageTableLevel {
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}

/// WARNING: This implementation will work as long as the P4 table follows
/// recursive page mapping.

impl<L> PageTable<L> where L: PageTableHierarchicalLevel {
    pub unsafe fn next_table_address_in_active(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();
        if entry_flags.contains(PRESENT) && !entry_flags.contains(HUGE_PAGE) {
            let table_address = self as *const _ as usize;
            Some((table_address << 9) | (index << 12))
        } else {
            None
        }
    }

    /// WARNING: Only works for active table.

    pub unsafe fn next_table_in_active(&self, index: usize) -> Option<&PageTable<L::NextLevel>> {
        self.next_table_address_in_active(index)
            .map(|address| unsafe { &*(address as *const _) })
    }

    /// WARNING: Only works for active table.

    pub unsafe fn next_table_mut_in_active(&mut self, index: usize) -> Option<&mut PageTable<L::NextLevel>> {
        self.next_table_address_in_active(index)
            .map(|address| unsafe { &mut *(address as *mut _) })
    }

    pub fn next_table_create<F>(&mut self, index: usize, untyped: &mut UntypedCapability, mapper: &mut Mapper, f: F)
        where F: FnOnce(&mut PageTable<L::NextLevel>, &mut UntypedCapability, &mut Mapper) {
        if self[index].is_unused() {
            let mut block = untyped.retype(PAGE_SIZE, PAGE_SIZE);

            self[index].set_address(block.start_addr(), PRESENT | WRITABLE);

            unsafe {
                mapper.borrow_mut_map(block.start_addr(), 1, |next_table : &mut PageTable<L::NextLevel>, mapper| {
                    next_table.zero();
                    f(next_table, untyped, mapper);
                });
            }
        } else {
            unsafe {
                mapper.borrow_mut_map(self[index].physical_address().unwrap(), 1, |next_table : &mut PageTable<L::NextLevel>, mapper| {
                    f(next_table, untyped, mapper);
                })
            }
        }
    }
}
