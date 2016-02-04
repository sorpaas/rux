use common::*;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use super::entry::*;
use super::super::{MemoryBlockCapability};
use super::super::UntypedCapability;
use super::super::utils;

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
    fn next_table_address(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();
        if entry_flags.contains(PRESENT) && !entry_flags.contains(HUGE_PAGE) {
            let table_address = self as *const _ as usize;
            Some((table_address << 9) | (index << 12))
        } else {
            None
        }
    }

    pub fn next_table(&self, index: usize) -> Option<&PageTable<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &*(address as *const _) })
    }

    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut PageTable<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &mut *(address as *mut _) })
    }

    pub fn next_table_create(&mut self,
                             index: usize,
                             untyped: UntypedCapability) -> (&mut PageTable<L::NextLevel>, Option<UntypedCapability>) {
        if self.next_table(index).is_none() {
            assert!(!self.entries[index].flags().contains(HUGE_PAGE),
                    "mapping code does not support huge pages");

            let page_start_addr = utils::necessary_page_start_addr(untyped.block_start_addr());
            let block_size = utils::necessary_block_size(untyped.block_start_addr(), 1);
            let (mut u1, u2) = UntypedCapability::from_untyped(untyped, block_size);

            assert!(u1.block_size() == block_size, "No frames available.");
            self.entries[index].set_address(page_start_addr, PRESENT | WRITABLE);
            self.next_table_mut(index).unwrap().zero();

            u1.block_size = 0;

            return (self.next_table_mut(index).unwrap(), u2);
        } else {
            return (self.next_table_mut(index).unwrap(), Some(untyped));
        }
    }
}
