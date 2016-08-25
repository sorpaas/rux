mod multiboot;

use ::{kmain};
use super::{kernel_end_paddr, kernel_start_paddr};
use super::paging::{BASE_PAGE_LENGTH, PD};

use utils::{block_count, align_up};

use core::mem;
use core::slice;
use common::{PAddr, VAddr};

extern {
    static multiboot_sig: u32;
    static multiboot_ptr: u64;

    static mut init_pd: PD;

    static kernel_stack_guard_page: u64;
}

pub fn multiboot_paddr() -> PAddr {
    PAddr::from_u64(multiboot_ptr)
}

pub fn kernel_stack_guard_page_vaddr() -> VAddr {
    VAddr::from_u64((&kernel_stack_guard_page as *const _) as u64)
}

pub fn init_pd_vaddr() -> VAddr {
    unsafe { VAddr::from_u64((&init_pd as *const _) as u64) }
}

#[derive(Debug)]
pub struct ArchInfo {
    free_memory_length: usize,
    free_memory_regions: [Option<MemoryRegion>; 16]
}

impl ArchInfo {
    pub fn free_memory_regions(&self) -> &[Option<MemoryRegion>; 16] {
        &self.free_memory_regions
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryRegion {
    start_paddr: PAddr,
    length: usize
}

impl MemoryRegion {
    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn move_up(&mut self, npaddr: PAddr) {
        assert!(npaddr > self.start_paddr);
        assert!(self.start_paddr + self.length > npaddr);
        let nlength = self.start_paddr.as_usize() + self.length - npaddr.as_usize();
        self.length = nlength;
        self.start_paddr = npaddr;
    }
}

/// Kernel entrypoint
#[lang="start"]
#[no_mangle]
pub fn kinit() {
    let kernel_page_size = block_count(kernel_end_paddr().as_usize() - kernel_start_paddr().as_usize(), BASE_PAGE_LENGTH);

    log!("kernel_page_size: {}", kernel_page_size);
        
    let bootinfo = unsafe {
        multiboot::Multiboot::new(multiboot_paddr(), |addr, size| {
            let ptr = mem::transmute(super::kernel_paddr_to_vaddr(addr).as_usize());
            Some(slice::from_raw_parts(ptr, size))
        })
    }.unwrap();

    log!("multiboot: {:?}", bootinfo);

    let mut archinfo = ArchInfo { free_memory_length: 0,
                                  free_memory_regions: [None; 16] };
    let mut alloc_region: Option<MemoryRegion> = None;
    
    for area in bootinfo.memory_regions().unwrap() {
        use self::multiboot::{MemoryType};
        
        if !(area.memory_type() == MemoryType::RAM) {
            continue;
        }

        let mut cur_region = MemoryRegion {
            start_paddr: area.base_address(),
            length: area.length() as usize
        };

        if cur_region.start_paddr <= kernel_start_paddr() &&
            PAddr::from_usize(cur_region.start_paddr.as_usize() + cur_region.length) >= kernel_end_paddr()
        {
            let npaddr = align_up(kernel_end_paddr(), BASE_PAGE_LENGTH);
            cur_region.move_up(npaddr);

            if alloc_region.is_none() {
                alloc_region = Some(cur_region);
            }
        } else {
            archinfo.free_memory_regions[archinfo.free_memory_length] = Some(cur_region);
        }
    }

    log!("archinfo: {:?}", archinfo);
    log!("alloc_region: {:?}", alloc_region);

    kmain();
}
