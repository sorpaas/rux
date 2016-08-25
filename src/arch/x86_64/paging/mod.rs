use core::fmt;
use common::PAddr;

#[macro_use]
mod macros;

mod table;

pub const BASE_PAGE_LENGTH: usize = 4096; // 4 KiB
pub const LARGE_PAGE_LENGTH: usize = 1024 * 1024 * 2; // 2 MiB
pub const HUGE_PAGE_LENGTH: usize = 1024 * 1024 * 1024; // 1 GiB
pub const CACHE_LINE_LENGTH: usize = 64; // 64 Bytes

/// MAXPHYADDR, which is at most 52; (use CPUID for finding system value).
pub const MAXPHYADDR: u64 = 52;

/// Mask to find the physical address of an entry in a page-table.
const ADDRESS_MASK: u64 = ((1 << MAXPHYADDR) - 1) & !0xfff;

pub use self::table::{PML4, PDPT, PD, PT, PML4Entry, PDPTEntry, PDEntry, PTEntry};
