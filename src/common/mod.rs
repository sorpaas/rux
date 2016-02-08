use core::mem::size_of;

pub type PhysicalAddress = usize;

pub const PAGE_TABLE_ENTRY_COUNT: usize = 512;
pub const CAPABILITY_POOL_COUNT: usize = 64;
pub const PAGE_SIZE: usize = 4096;
