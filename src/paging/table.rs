use common::*;
use memory::FrameAllocator;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use super::entry::*;

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
