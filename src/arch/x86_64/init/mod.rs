mod multiboot;

use ::{kmain};
use super::{kernel_end_paddr, kernel_start_paddr, KERNEL_BASE};
use super::paging::{BASE_PAGE_LENGTH, PD, PT, PML4, pd_index};
use self::multiboot::{Multiboot};

use utils::{block_count, align_up};

use core::mem;
use core::slice;
use core::ptr::{Unique};
use common::{PAddr, VAddr};

extern {
    static multiboot_sig: u32;
    static multiboot_ptr: u64;
    static mut init_pd: PD;
    static kernel_stack_guard_page: u64;
}

// Below should be used BEFORE switching to new page table structure.
const INITIAL_ALLOC_START_VADDR: VAddr = VAddr::from_u64(KERNEL_BASE + 0xc00000);
const INITIAL_ALLOC_PML4_VADDR: VAddr = VAddr::from_u64(KERNEL_BASE + 0xc00000);
const INITIAL_ALLOC_PDPT_VADDR: VAddr = VAddr::from_u64(KERNEL_BASE + 0xc01000);
const INITIAL_ALLOC_PD_VADDR: VAddr = VAddr::from_u64(KERNEL_BASE + 0xc02000);
const INITIAL_ALLOC_OBJECT_POOL_PT_VADDR: VAddr = VAddr::from_u64(KERNEL_BASE + 0xc03000);
const INITIAL_ALLOC_KERNEL_PT_START_VADDR: VAddr = VAddr::from_u64(KERNEL_BASE + 0xc04000);

// Below should be used AFTER switching to new page table structure.
const OBJECT_POOL_START_VADDR: VAddr = VAddr::from_u64(KERNEL_BASE + 0xe00000);
const OBJECT_POOL_SIZE: usize = 511;
const OBJECT_POOL_PT_VADDR: VAddr = VAddr::from_u64(KERNEL_BASE + 0xfff000);

// Variables
global_variable!(initial_pd, initial_pd_mut, _initial_pd, PD, unsafe { Some(Unique::new(&mut init_pd as *mut _)) });
global_variable!(object_pool_pt, object_pool_pt_mut, _object_pool_pt, PT, None);

global_const!(kernel_pml4_paddr, _kernel_pml4_paddr, PAddr, None);
global_const!(kernel_pd_paddr, _kernel_pd_paddr, PD, None);

// Helper functions
pub fn multiboot_paddr() -> PAddr {
    PAddr::from_u64(multiboot_ptr)
}

pub fn kernel_stack_guard_page_vaddr() -> VAddr {
    VAddr::from_u64((&kernel_stack_guard_page as *const _) as u64)
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
        assert!(npaddr >= self.start_paddr);
        assert!(self.start_paddr + self.length > npaddr);
        let nlength = self.start_paddr.as_usize() + self.length - npaddr.as_usize();
        self.length = nlength;
        self.start_paddr = npaddr;
    }
}

// fn alloc_kernel_pml4(kernel_pdpt: &PDPT, &mut MemoryRegion) -> Unique<PML4> {

// }

// fn alloc_kernel_pdpt(kernel_pd: &PD, &mut MemoryRegion) -> Unique<PDPT> {

// }

// fn alloc_kernel_pd(kernel_pts: &[&PT], &mut MemoryRegion) -> Unique<PD> {

// }

// fn alloc_kernel_pts(&mut MemoryRegion) -> &[Unique<PT>] {
    
// }

// fn map_alloc_in_init_pd(alloc_start_vaddr: VAddr, alloc_start_paddr: PAddr, init_pd_mut: &mut PD) {

// }

fn bootstrap_archinfo() -> (ArchInfo, MemoryRegion) {
    let bootinfo = unsafe {
        multiboot::Multiboot::new(multiboot_paddr(), |addr, size| {
            let ptr = mem::transmute(super::kernel_paddr_to_vaddr(addr).as_usize());
            Some(slice::from_raw_parts(ptr, size))
        })
    }.unwrap();
    
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

    (archinfo, alloc_region.unwrap())
}

// This maps 2MB for allocation region
fn map_alloc_region(alloc_region: &mut MemoryRegion) {
    use super::paging::{PD_P, PD_RW, PD_PS, PDEntry};
    
    let map_alloc_start_vaddr = INITIAL_ALLOC_START_VADDR;
    let map_alloc_pd_index = pd_index(map_alloc_start_vaddr);
    let map_alloc_start_paddr = align_up(alloc_region.start_paddr(), BASE_PAGE_LENGTH);

    initial_pd_mut()[map_alloc_pd_index] = PDEntry::new(map_alloc_start_paddr, PD_P | PD_RW | PD_PS);

    unsafe { super::paging::flush(map_alloc_start_vaddr); }
}

/// Kernel entrypoint
#[lang="start"]
#[no_mangle]
pub fn kinit() {
    use super::paging::{PDEntry, PD_P, PD_RW, PD_PS, BASE_PAGE_LENGTH};
    
    let kernel_page_size = block_count(kernel_end_paddr().as_usize() - kernel_start_paddr().as_usize(), BASE_PAGE_LENGTH);
    let alloc_size =
        1 + // One PML4
        1 + // One PDPT
        1 + // One PD
        block_count(kernel_page_size, 512) + // Kernel page mapping PT
        1
    // The object pool PT, with its last item pointing to itself at address
    // KERNEL_BASE + 0xfff000
        ;

    assert!(alloc_size <= 512);

    let (mut archinfo, mut alloc_region) = bootstrap_archinfo();
    
    log!("archinfo: {:?}", archinfo);
    log!("alloc_region: {:?}", alloc_region);

    // Before allocation, we need to make sure PAddr + alloc_size is mapped. This is done in the init_pd.

    assert!(alloc_size * BASE_PAGE_LENGTH < alloc_region.length());
    map_alloc_region(&mut alloc_region);

    kmain();
}
