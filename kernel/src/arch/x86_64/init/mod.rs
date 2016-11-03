mod multiboot;
mod paging;
mod interrupt;
mod segmentation;

pub use self::paging::{KERNEL_PML4, KERNEL_PDPT, KERNEL_PD,
                       OBJECT_POOL_PT, OBJECT_POOL_START_VADDR};
pub use self::segmentation::{set_kernel_stack};

use ::{kmain};
use super::{kernel_end_paddr, kernel_start_paddr, kernel_start_vaddr, kernel_end_vaddr, KERNEL_BASE};
use super::paging::{BASE_PAGE_LENGTH, LARGE_PAGE_LENGTH,
                    PD, PT, PML4, PDPT, PTEntry, pd_index, pml4_index, pt_index, pdpt_index};
use self::multiboot::{Multiboot};

use util::{block_count, align_up};

use core::mem;
use core::slice::{self, Iter};
use core::ptr::{Unique};

use common::{PAddr, VAddr, MemoryRegion};

extern {
    static multiboot_sig: u32;
    static multiboot_ptr: u64;
}

// Helper functions
pub fn multiboot_paddr() -> PAddr {
    PAddr::from(multiboot_ptr)
}

pub struct FreeRegionsIterator<'a>(Iter<'a, Option<MemoryRegion>>);

impl<'a> Iterator for FreeRegionsIterator<'a> {
    type Item = MemoryRegion;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.0.next();

        if item.is_none() {
            None
        } else {
            if item.unwrap().is_none() {
                None
            } else {
                Some(item.unwrap().unwrap())
            }
        }
    }
}

#[derive(Debug)]
pub struct InitInfo {
    free_regions_size: usize,
    free_regions: [Option<MemoryRegion>; 16],
    rinit_region: MemoryRegion,
    kernel_region: MemoryRegion,
}

impl InitInfo {
    pub fn free_regions(&self) -> FreeRegionsIterator {
        FreeRegionsIterator(self.free_regions.iter())
    }

    pub fn kernel_region(&self) -> MemoryRegion {
        self.kernel_region
    }

    pub fn rinit_region(&self) -> MemoryRegion {
        self.rinit_region
    }

    pub fn new(kernel_region: MemoryRegion, rinit_region: MemoryRegion) -> InitInfo {
        InitInfo { free_regions_size: 0,
                   free_regions: [None; 16],
                   kernel_region: kernel_region,
                   rinit_region: rinit_region }
    }

    pub fn push_free_region(&mut self, region: MemoryRegion) {
        self.free_regions[self.free_regions_size] = Some(region);
        self.free_regions_size += 1;
    }
}

fn bootstrap_archinfo() -> (InitInfo, MemoryRegion) {
    let bootinfo = unsafe {
        multiboot::Multiboot::new(multiboot_paddr(), |addr, size| {
            let ptr = mem::transmute(super::kernel_paddr_to_vaddr(addr).into(): usize);
            Some(slice::from_raw_parts(ptr, size))
        })
    }.unwrap();

    log!("bootinfo: {:?}", bootinfo);

    let rinit_module = bootinfo.modules().unwrap().next().unwrap();
    log!("rinit module: {:?}", rinit_module);
    
    let mut archinfo = InitInfo::new(
        MemoryRegion::new(kernel_start_paddr(),
                          kernel_end_paddr().into(): usize + 1 -
                          kernel_start_paddr().into(): usize),
        MemoryRegion::new(rinit_module.start,
                          rinit_module.end.into(): usize + 1 -
                          rinit_module.start.into(): usize));
    let mut alloc_region: Option<MemoryRegion> = None;
    
    for area in bootinfo.memory_regions().unwrap() {
        use self::multiboot::{MemoryType};
        
        if !(area.memory_type() == MemoryType::RAM) {
            continue;
        }

        let mut cur_region = MemoryRegion::new(area.base_address(), area.length() as usize);

        if cur_region.skip_up(&archinfo.kernel_region()) {
            assert!(cur_region.skip_up(&archinfo.rinit_region()));
            alloc_region = Some(cur_region);
        } else {
            archinfo.push_free_region(cur_region);
        }
    }

    (archinfo, alloc_region.unwrap())
}

/// Kernel entrypoint
#[lang="start"]
#[no_mangle]
pub fn kinit() {
    let (mut archinfo, mut alloc_region) = bootstrap_archinfo();

    log!("kernel_start_vaddr: 0x{:x}", kernel_start_vaddr());
    log!("archinfo: {:?}", archinfo);
    log!("alloc_region: {:?}", alloc_region);

    paging::init(&mut alloc_region);
    segmentation::init();
    interrupt::init();

    archinfo.push_free_region(alloc_region);
    kmain(archinfo);
}
