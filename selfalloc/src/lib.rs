#![feature(const_fn)]
#![feature(global_allocator)]
#![feature(alloc)]
#![feature(allocator_api)]
#![no_std]

extern crate system;
extern crate spin;
extern crate alloc;
extern crate abi;

use spin::{Mutex};
use abi::{CAddr};
use core::ops::{Deref, DerefMut};

const PAGE_LENGTH: usize = 4096;

static ALLOCATOR: Mutex<Option<WatermarkAllocator>> = Mutex::new(None);

struct WatermarkAllocator {
    untyped_cap: CAddr,
    pt_cap: CAddr,
    page_cap: Option<CAddr>,
    page_start_addr: usize,
    watermark: usize,
}

pub unsafe fn setup_allocator(untyped_cap: CAddr, pt_cap: CAddr, page_start_addr: usize) {
    *ALLOCATOR.lock() = Some(WatermarkAllocator::new(untyped_cap, pt_cap, page_start_addr));
}

// http://os.phil-opp.com/kernel-heap.html#alignment

/// Align downwards. Returns the greatest x with alignment `align`
/// so that x <= addr. The alignment must be a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

/// Align upwards. Returns the smallest x with alignment `align`
/// so that x >= addr. The alignment must be a power of 2.
pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}

impl WatermarkAllocator {
    const fn new(untyped_cap: CAddr, pt_cap: CAddr, page_start_addr: usize) -> Self {
        WatermarkAllocator {
            untyped_cap: untyped_cap,
            pt_cap: pt_cap,
            page_cap: None,
            page_start_addr: page_start_addr,
            watermark: 0,
        }
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> *mut u8 {
        let alloc_start = align_up(self.watermark, align);
        let alloc_end = alloc_start.saturating_add(size);

        if alloc_end >= PAGE_LENGTH {
            self.page_cap = None;
            self.page_start_addr += PAGE_LENGTH;
            self.watermark = 0;

            return self.allocate(size, align);
        }

        if self.page_cap.is_none() {
            self.page_cap = Some(system::retype_raw_page_free(self.untyped_cap));
            system::map_raw_page_free(self.untyped_cap, self.pt_cap, self.page_cap.clone().unwrap(), self.page_start_addr);

            return self.allocate(size, align);
        }

        self.watermark = alloc_end;
        (self.page_start_addr + alloc_start) as *mut u8
    }
}

#[global_allocator]
static WATER_ALLOCATOR: WaterAlloc = WaterAlloc;

use alloc::heap::Alloc;
use alloc::allocator::{AllocErr, Layout};

struct WaterAlloc;

unsafe impl<'a> Alloc for &'a WaterAlloc {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        Ok(ALLOCATOR.lock().as_mut().unwrap().allocate(layout.size(), layout.align()))
    }

    unsafe fn dealloc(&mut self, pointer: *mut u8, layout: Layout) { }
}
