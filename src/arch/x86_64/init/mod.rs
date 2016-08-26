mod multiboot;
mod archinfo;

pub use self::archinfo::{ArchInfo, MemoryRegion};

use ::{kmain};
use super::{kernel_end_paddr, kernel_start_paddr, kernel_start_vaddr, kernel_end_vaddr, KERNEL_BASE};
use super::paging::{BASE_PAGE_LENGTH, LARGE_PAGE_LENGTH,
                    PD, PT, PML4, PDPT, PTEntry, pd_index, pml4_index, pt_index, pdpt_index};
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
const INITIAL_ALLOC_PML4_OFFSET: usize = 0x0000;
const INITIAL_ALLOC_PDPT_OFFSET: usize = 0x1000;
const INITIAL_ALLOC_PD_OFFSET: usize = 0x2000;
const INITIAL_ALLOC_OBJECT_POOL_PT_OFFSET: usize = 0x3000;
const INITIAL_ALLOC_KERNEL_PT_START_OFFSET: usize = 0x4000;

// Below should be used AFTER switching to new page table structure.
pub const OBJECT_POOL_START_VADDR: VAddr = VAddr::from_u64(KERNEL_BASE + 0xe00000);
pub const OBJECT_POOL_SIZE: usize = 511;
pub const OBJECT_POOL_PT_VADDR: VAddr = VAddr::from_u64(KERNEL_BASE + 0xfff000);

// Variables
global_variable!(initial_pd, initial_pd_mut, _initial_pd, PD, unsafe { Some(Unique::new(&mut init_pd as *mut _)) });
global_variable!(object_pool_pt, object_pool_pt_mut, _object_pool_pt, [PTEntry; OBJECT_POOL_SIZE], None);

global_const!(kernel_pml4_paddr, _kernel_pml4_paddr, PAddr, None);
global_const!(kernel_pdpt_paddr, _kernel_pdpt_paddr, PAddr, None);
global_const!(kernel_pd_paddr, _kernel_pd_paddr, PAddr, None);

// Helper functions
pub fn multiboot_paddr() -> PAddr {
    PAddr::from_u64(multiboot_ptr)
}

pub fn kernel_stack_guard_page_vaddr() -> VAddr {
    VAddr::from_u64((&kernel_stack_guard_page as *const _) as u64)
}

fn alloc_kernel_pml4(region: &mut MemoryRegion, alloc_base: PAddr) -> Unique<PML4> {
    use super::paging::PML4Entry;
    
    let paddr = alloc_base + INITIAL_ALLOC_PML4_OFFSET;
    let vaddr = INITIAL_ALLOC_START_VADDR + INITIAL_ALLOC_PML4_OFFSET;

    log!("pml4, paddr: 0x{:x}, vaddr: 0x{:x}", paddr, vaddr);

    let mut pml4_unique = unsafe { Unique::new(vaddr.as_usize() as *mut PML4) };

    {
        let mut pml4 = unsafe { pml4_unique.get_mut() };
        *pml4 = [PML4Entry::empty(); 512];
    }

    region.move_up(paddr + BASE_PAGE_LENGTH);

    unsafe { _kernel_pml4_paddr = Some(paddr); }
    
    pml4_unique
}

fn alloc_kernel_pdpt(region: &mut MemoryRegion, pml4: &mut PML4, alloc_base: PAddr) -> Unique<PDPT> {
    use super::paging::{PDPTEntry, PML4Entry, PML4_P, PML4_RW};
    
    let paddr = alloc_base + INITIAL_ALLOC_PDPT_OFFSET;
    let vaddr = INITIAL_ALLOC_START_VADDR + INITIAL_ALLOC_PDPT_OFFSET;

    log!("pdpt, paddr: 0x{:x}, vaddr: 0x{:x}", paddr, vaddr);

    let mut pdpt_unique = unsafe { Unique::new(vaddr.as_usize() as *mut PDPT) };

    {
        let mut pdpt = unsafe { pdpt_unique.get_mut() };
        *pdpt = [PDPTEntry::empty(); 512];
    }

    region.move_up(paddr + BASE_PAGE_LENGTH);
    
    pml4[pml4_index(VAddr::from_u64(KERNEL_BASE))] = PML4Entry::new(paddr, PML4_P | PML4_RW);

    unsafe { _kernel_pdpt_paddr = Some(paddr); }

    pdpt_unique
}

fn alloc_kernel_pd(region: &mut MemoryRegion, pdpt: &mut PDPT, alloc_base: PAddr) -> Unique<PD> {
    use super::paging::{PDEntry, PDPTEntry, PDPT_P, PDPT_RW};
    
    let paddr = alloc_base + INITIAL_ALLOC_PD_OFFSET;
    let vaddr = INITIAL_ALLOC_START_VADDR + INITIAL_ALLOC_PD_OFFSET;

    log!("pd, paddr: 0x{:x}, vaddr: 0x{:x}", paddr, vaddr);

    let mut pd_unique = unsafe { Unique::new(vaddr.as_usize() as *mut PD) };

    {
        let mut pd = unsafe { pd_unique.get_mut() };
        *pd = [PDEntry::empty(); 512];
    }

    region.move_up(paddr + BASE_PAGE_LENGTH);

    pdpt[pdpt_index(VAddr::from_u64(KERNEL_BASE))] = PDPTEntry::new(paddr, PDPT_P | PDPT_RW);

    unsafe { _kernel_pd_paddr = Some(paddr); }

    pd_unique
}

fn alloc_object_pool_pt(region: &mut MemoryRegion, pd: &mut PD, alloc_base: PAddr) -> Unique<PT> {
    use super::paging::{PTEntry, PDEntry, PD_P, PD_RW, PT_P, PT_RW};
    
    let paddr = alloc_base + INITIAL_ALLOC_OBJECT_POOL_PT_OFFSET;
    let vaddr = INITIAL_ALLOC_START_VADDR + INITIAL_ALLOC_OBJECT_POOL_PT_OFFSET;

    log!("object_pool_pt, paddr: 0x{:x}, vaddr: 0x{:x}", paddr, vaddr);

    let mut pt_unique = unsafe { Unique::new(vaddr.as_usize() as *mut PT) };

    {
        let mut pt = unsafe { pt_unique.get_mut() };
        *pt = [PTEntry::empty(); 512];

        let reverse_pt_index = pt_index(OBJECT_POOL_PT_VADDR);
        log!("reverse_pt_index: {}", reverse_pt_index);

        pt[reverse_pt_index] = PTEntry::new(paddr, PT_P | PT_RW);
    }

    region.move_up(paddr + BASE_PAGE_LENGTH);

    pd[pd_index(OBJECT_POOL_START_VADDR)] = PDEntry::new(paddr, PD_P | PD_RW);

    pt_unique
}

fn alloc_kernel_page(pt: &mut PT, offset_size: usize, alloc_base: PAddr) {
    use super::paging::{PT_P, PT_RW};
    
    let paddr = kernel_start_paddr() + (offset_size * BASE_PAGE_LENGTH);
    let vaddr = kernel_start_vaddr() + (offset_size * BASE_PAGE_LENGTH);

    log!("kernel page allocated at 0x{:x}", vaddr);

    pt[pt_index(vaddr)] = PTEntry::new(paddr, PT_P | PT_RW);
}

fn alloc_kernel_guard_page(pt: &mut PT, offset_size: usize, alloc_base: PAddr) {
    use super::paging::{PT_P, PT_RW};
    
    let paddr = kernel_start_paddr() + (offset_size * BASE_PAGE_LENGTH);
    let vaddr = kernel_start_vaddr() + (offset_size * BASE_PAGE_LENGTH);

    log!("guard page allocated at 0x{:x}", vaddr);

    pt[pt_index(vaddr)] = PTEntry::empty();
}

fn alloc_kernel_pts(region: &mut MemoryRegion, pd: &mut PD, alloc_base: PAddr) {
    use super::paging::{PDEntry, PD_P, PD_RW};
    
    let kernel_page_size = block_count(kernel_end_paddr().as_usize() - kernel_start_paddr().as_usize(), BASE_PAGE_LENGTH);
    let guard_page_index = (kernel_stack_guard_page_vaddr().as_usize() - kernel_start_vaddr().as_usize()) / BASE_PAGE_LENGTH;

    log!("guard_page_index: {}", guard_page_index);

    for i in 0..kernel_page_size {
        if i % 512 == 0 {
            pd[pd_index(kernel_start_vaddr() + i * BASE_PAGE_LENGTH)] = PDEntry::new(region.start_paddr(), PD_P | PD_RW);
            let npaddr = region.start_paddr() + BASE_PAGE_LENGTH;
            region.move_up(npaddr);
        }

        let pt_entry = pd[pd_index(kernel_start_vaddr() + i * BASE_PAGE_LENGTH)];

        let offset = pt_entry.get_address().as_usize() - alloc_base.as_usize();
        let mut pt_unique = unsafe {
            Unique::new((INITIAL_ALLOC_START_VADDR + offset).as_usize() as *mut PT) };
        
        if i == guard_page_index {
            alloc_kernel_guard_page(unsafe { pt_unique.get_mut() }, i % 512, alloc_base);
        } else {
            alloc_kernel_page(unsafe { pt_unique.get_mut() }, i % 512, alloc_base);
        }
    }
}

fn bootstrap_archinfo() -> (ArchInfo, MemoryRegion) {
    let bootinfo = unsafe {
        multiboot::Multiboot::new(multiboot_paddr(), |addr, size| {
            let ptr = mem::transmute(super::kernel_paddr_to_vaddr(addr).as_usize());
            Some(slice::from_raw_parts(ptr, size))
        })
    }.unwrap();
    
    let mut archinfo = ArchInfo::new();
    let mut alloc_region: Option<MemoryRegion> = None;
    
    for area in bootinfo.memory_regions().unwrap() {
        use self::multiboot::{MemoryType};
        
        if !(area.memory_type() == MemoryType::RAM) {
            continue;
        }

        let mut cur_region = MemoryRegion::new(area.base_address(), area.length() as usize);

        if cur_region.start_paddr() <= kernel_start_paddr() &&
            PAddr::from_usize(cur_region.start_paddr().as_usize() +
                              cur_region.length()) >= kernel_end_paddr()
        {
            let npaddr = align_up(kernel_end_paddr(), BASE_PAGE_LENGTH);
            cur_region.move_up(npaddr);
            
            if alloc_region.is_none() {
                alloc_region = Some(cur_region);
            }
        } else {
            archinfo.push_memory_region(cur_region);
        }
    }

    (archinfo, alloc_region.unwrap())
}

// This maps 2MB for allocation region
fn map_alloc_region(alloc_region: &mut MemoryRegion) -> PAddr {
    use super::paging::{PD_P, PD_RW, PD_PS, PDEntry};
    
    let map_alloc_start_vaddr = INITIAL_ALLOC_START_VADDR;
    let map_alloc_pd_index = pd_index(map_alloc_start_vaddr);
    let map_alloc_start_paddr = align_up(alloc_region.start_paddr(), LARGE_PAGE_LENGTH);

    initial_pd_mut()[map_alloc_pd_index] = PDEntry::new(map_alloc_start_paddr, PD_P | PD_RW | PD_PS);

    // unsafe { super::paging::flush(map_alloc_start_vaddr); }
    unsafe { super::paging::flush_all(); }

    alloc_region.move_up(map_alloc_start_paddr);
    
    map_alloc_start_paddr
}

/// Kernel entrypoint
#[lang="start"]
#[no_mangle]
pub fn kinit() {
    use super::paging::{PDEntry, PD_P, PD_RW, PD_PS, BASE_PAGE_LENGTH};
    use x86::shared::control_regs;
    
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

    log!("kernel_start_vaddr: 0x{:x}", kernel_start_vaddr());
    log!("archinfo: {:?}", archinfo);
    log!("alloc_region: {:?}", alloc_region);

    // Before allocation, we need to make sure PAddr + alloc_size is mapped. This is done in the init_pd.

    assert!(alloc_size * BASE_PAGE_LENGTH < alloc_region.length());
    let alloc_base_paddr = map_alloc_region(&mut alloc_region);

    log!("alloc_base_paddr: 0x{:x}", alloc_base_paddr);

    let mut pml4_unique = alloc_kernel_pml4(&mut alloc_region, alloc_base_paddr);
    let mut pdpt_unique = alloc_kernel_pdpt(&mut alloc_region, unsafe { pml4_unique.get_mut() }, alloc_base_paddr);
    let mut pd_unique = alloc_kernel_pd(&mut alloc_region, unsafe { pdpt_unique.get_mut() }, alloc_base_paddr);
    let mut object_pool_pt_unique = alloc_object_pool_pt(&mut alloc_region, unsafe { pd_unique.get_mut() }, alloc_base_paddr);

    alloc_kernel_pts(&mut alloc_region, unsafe { pd_unique.get_mut() }, alloc_base_paddr);

    unsafe {
        _initial_pd = None;
    }
    unsafe { super::paging::switch_to(kernel_pml4_paddr()); }
    unsafe {
        _object_pool_pt = Some(Unique::new(OBJECT_POOL_PT_VADDR.as_usize() as *mut _));
    }

    archinfo.push_memory_region(alloc_region);

    kmain(archinfo);
}
