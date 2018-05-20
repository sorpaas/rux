#![feature(const_fn)]
#![feature(global_allocator)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(lang_items)]
#![no_std]

#[macro_use]
extern crate system;
extern crate spin;
extern crate alloc;
extern crate abi;

use spin::{Once, Mutex};
use abi::CAddr;

const PAGE_LENGTH: usize = 4096;

static ALLOCATOR: Once<Mutex<WatermarkAllocator>> = Once::new();

struct WatermarkAllocator {
    untyped_cap: CAddr,
    toplevel_table_cap: CAddr,
    page_cap: CAddr,
    page_start_addr: usize,
    watermark: usize,
}

pub unsafe fn setup_allocator(untyped_cap: CAddr, pt_cap: CAddr, page_start_addr: usize) {
    ALLOCATOR.call_once(|| Mutex::new(WatermarkAllocator::new(untyped_cap, pt_cap, page_start_addr)));
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
    fn new(untyped_cap: CAddr, toplevel_table_cap: CAddr, page_start_addr: usize) -> Self {
        let page_cap = system::retype_raw_page_free(untyped_cap);
        system::map_raw_page_free(page_start_addr, untyped_cap, toplevel_table_cap, page_cap.clone());

        WatermarkAllocator {
            untyped_cap: untyped_cap,
            page_cap: page_cap,
            toplevel_table_cap: toplevel_table_cap,
            page_start_addr: page_start_addr,
            watermark: 0,
        }
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> *mut u8 {
        let alloc_start = align_up(self.watermark, align);
        let ret = (self.page_start_addr + alloc_start) as *mut u8;

        let mut alloc_end = alloc_start.saturating_add(size);

        while alloc_end >= PAGE_LENGTH {
            self.page_cap = system::retype_raw_page_free(self.untyped_cap);
            self.page_start_addr += PAGE_LENGTH;
            system::map_raw_page_free(self.page_start_addr, self.untyped_cap, self.toplevel_table_cap, self.page_cap.clone());

            alloc_end -= PAGE_LENGTH;
        }

        self.watermark = alloc_end;
        ret
    }
}

#[global_allocator]
static WATER_ALLOCATOR: WaterAlloc = WaterAlloc;

use alloc::allocator::Layout;
use core::alloc::{GlobalAlloc, Opaque};

struct WaterAlloc;

unsafe impl<'a> GlobalAlloc for WaterAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        ALLOCATOR.wait().unwrap().lock().allocate(layout.size(), layout.align()) as _
    }

    unsafe fn dealloc(&self, _pointer: *mut Opaque, _layout: Layout) { }
}

#[lang="oom"]
#[no_mangle]
pub fn rust_oom() -> ! {
    panic!("Out of memory");
}
