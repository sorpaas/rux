use core::ptr::{Unique};

use arch::{kernel_start_paddr, kernel_start_vaddr,
           kernel_end_paddr, kernel_end_vaddr};
use arch::paging::{PTEntry, PML4, PDPT, PD, PT,
                   pml4_index, pdpt_index, pd_index, pt_index,
                   BASE_PAGE_LENGTH, LARGE_PAGE_LENGTH};
use arch::{KERNEL_BASE};
use common::{PAddr, VAddr, MemoryRegion};
use util::{block_count, align_up, ExternReadonlyObject, ExternMutex};
use core::ops::{Deref};

use arch::addr;

extern {
    static mut init_pd: PD;
    static kernel_stack_guard_page: u64;
}

// Below should be used BEFORE switching to new page table structure.
const INITIAL_ALLOC_START_VADDR: VAddr = VAddr::new(KERNEL_BASE + 0xc00000);
const INITIAL_ALLOC_PML4_OFFSET: usize = 0x0000;
const INITIAL_ALLOC_PDPT_OFFSET: usize = 0x1000;
const INITIAL_ALLOC_PD_OFFSET: usize = 0x2000;
const INITIAL_ALLOC_OBJECT_POOL_PT_OFFSET: usize = 0x3000;
const INITIAL_ALLOC_KERNEL_PT_START_OFFSET: usize = 0x4000;

// Below should be used AFTER switching to new page table structure.
pub const OBJECT_POOL_START_VADDR: VAddr = VAddr::new(KERNEL_BASE + 0xe00000);
pub const OBJECT_POOL_SIZE: usize = 510;
pub const OBJECT_POOL_PT_VADDR: VAddr = VAddr::new(KERNEL_BASE + 0xfff000);
pub const APIC_PAGE_VADDR: VAddr = VAddr::new(KERNEL_BASE + 0xffe000);

// Variables
static INITIAL_PD: ExternMutex<PD> =
    unsafe { ExternMutex::new(Some(&init_pd as *const _)) };

pub static OBJECT_POOL_PT: ExternMutex<[PTEntry; OBJECT_POOL_SIZE]> =
    unsafe { ExternMutex::new(None) };
pub static KERNEL_PML4: ExternReadonlyObject<PML4> =
    unsafe { ExternReadonlyObject::new() };
pub static KERNEL_PDPT: ExternReadonlyObject<PDPT> =
    unsafe { ExternReadonlyObject::new() };
pub static KERNEL_PD: ExternReadonlyObject<PD> =
    unsafe { ExternReadonlyObject::new() };

fn kernel_stack_guard_page_vaddr() -> VAddr {
    VAddr::from((&kernel_stack_guard_page as *const _) as u64)
}

fn alloc_kernel_pml4(region: &mut MemoryRegion, alloc_base: PAddr) -> Unique<PML4> {
    use arch::paging::PML4Entry;
    
    let paddr = alloc_base + INITIAL_ALLOC_PML4_OFFSET;
    let vaddr = INITIAL_ALLOC_START_VADDR + INITIAL_ALLOC_PML4_OFFSET;

    log!("pml4, paddr: 0x{:x}, vaddr: 0x{:x}", paddr, vaddr);

    let mut pml4_unique = unsafe { Unique::new(vaddr.into(): usize as *mut PML4) };

    {
        let mut pml4 = unsafe { pml4_unique.get_mut() };
        *pml4 = [PML4Entry::empty(); 512];
    }

    region.move_up(paddr + BASE_PAGE_LENGTH);

    unsafe { KERNEL_PML4.bootstrap(*pml4_unique.deref(), paddr); }
    
    pml4_unique
}

fn alloc_kernel_pdpt(region: &mut MemoryRegion, pml4: &mut PML4, alloc_base: PAddr) -> Unique<PDPT> {
    use arch::paging::{PDPTEntry, PML4Entry, PML4_P, PML4_RW};
    
    let paddr = alloc_base + INITIAL_ALLOC_PDPT_OFFSET;
    let vaddr = INITIAL_ALLOC_START_VADDR + INITIAL_ALLOC_PDPT_OFFSET;

    log!("pdpt, paddr: 0x{:x}, vaddr: 0x{:x}", paddr, vaddr);

    let mut pdpt_unique = unsafe { Unique::new(vaddr.into(): usize as *mut PDPT) };

    {
        let mut pdpt = unsafe { pdpt_unique.get_mut() };
        *pdpt = [PDPTEntry::empty(); 512];
    }

    region.move_up(paddr + BASE_PAGE_LENGTH);
    
    pml4[pml4_index(VAddr::from(KERNEL_BASE))] = PML4Entry::new(paddr, PML4_P | PML4_RW);

    unsafe { KERNEL_PDPT.bootstrap(*pdpt_unique.deref(), paddr) }

    pdpt_unique
}

fn alloc_kernel_pd(region: &mut MemoryRegion, pdpt: &mut PDPT, alloc_base: PAddr) -> Unique<PD> {
    use arch::paging::{PDEntry, PDPTEntry, PDPT_P, PDPT_RW};
    
    let paddr = alloc_base + INITIAL_ALLOC_PD_OFFSET;
    let vaddr = INITIAL_ALLOC_START_VADDR + INITIAL_ALLOC_PD_OFFSET;

    log!("pd, paddr: 0x{:x}, vaddr: 0x{:x}", paddr, vaddr);

    let mut pd_unique = unsafe { Unique::new(vaddr.into(): usize as *mut PD) };

    {
        let mut pd = unsafe { pd_unique.get_mut() };
        *pd = [PDEntry::empty(); 512];
    }

    region.move_up(paddr + BASE_PAGE_LENGTH);

    pdpt[pdpt_index(VAddr::from(KERNEL_BASE))] = PDPTEntry::new(paddr, PDPT_P | PDPT_RW);

    unsafe { KERNEL_PD.bootstrap(*pd_unique.deref(), paddr); }

    pd_unique
}

fn alloc_object_pool_pt(region: &mut MemoryRegion, pd: &mut PD, alloc_base: PAddr) -> Unique<PT> {
    use arch::paging::{PTEntry, PDEntry, PD_P, PD_RW, PT_P, PT_RW, PT_PWT, PT_PCD};
    
    let paddr = alloc_base + INITIAL_ALLOC_OBJECT_POOL_PT_OFFSET;
    let vaddr = INITIAL_ALLOC_START_VADDR + INITIAL_ALLOC_OBJECT_POOL_PT_OFFSET;

    log!("object_pool_pt, paddr: 0x{:x}, vaddr: 0x{:x}", paddr, vaddr);

    let mut pt_unique = unsafe { Unique::new(vaddr.into(): usize as *mut PT) };

    {
        let mut pt = unsafe { pt_unique.get_mut() };
        *pt = [PTEntry::empty(); 512];

        {
            // Mapping reverse ObjectPool PT Page
            let reverse_pt_index = pt_index(OBJECT_POOL_PT_VADDR);
            pt[reverse_pt_index] = PTEntry::new(paddr, PT_P | PT_RW);
        }

        {
            use x86::shared::msr;

            let apic_msr = unsafe { msr::rdmsr(0x1B) };
            let apic_base = PAddr::from((apic_msr >> 12) * 0x1000);
            // Mapping APIC Page
            let apic_pt_index = pt_index(APIC_PAGE_VADDR);
            log!("apic index: {}", apic_pt_index);
            pt[apic_pt_index] = PTEntry::new(apic_base, PT_P | PT_RW | PT_PWT | PT_PCD);
        }
    }

    region.move_up(paddr + BASE_PAGE_LENGTH);

    pd[pd_index(OBJECT_POOL_START_VADDR)] = PDEntry::new(paddr, PD_P | PD_RW);

    pt_unique
}

fn alloc_kernel_page(pt: &mut PT, offset_size: usize, alloc_base: PAddr) {
    use arch::paging::{PT_P, PT_RW};
    
    let paddr = kernel_start_paddr() + (offset_size * BASE_PAGE_LENGTH);
    let vaddr = kernel_start_vaddr() + (offset_size * BASE_PAGE_LENGTH);

    log!("kernel page allocated at 0x{:x}", vaddr);

    pt[pt_index(vaddr)] = PTEntry::new(paddr, PT_P | PT_RW);
}

fn alloc_kernel_guard_page(pt: &mut PT, offset_size: usize, alloc_base: PAddr) {
    use arch::paging::{PT_P, PT_RW};
    
    let paddr = kernel_start_paddr() + (offset_size * BASE_PAGE_LENGTH);
    let vaddr = kernel_start_vaddr() + (offset_size * BASE_PAGE_LENGTH);

    log!("guard page allocated at 0x{:x}", vaddr);

    pt[pt_index(vaddr)] = PTEntry::empty();
}

fn alloc_kernel_pts(region: &mut MemoryRegion, pd: &mut PD, alloc_base: PAddr) {
    use arch::paging::{PDEntry, PD_P, PD_RW};
    
    let kernel_page_size = block_count(kernel_end_paddr().into(): usize -
                                       kernel_start_paddr().into(): usize, BASE_PAGE_LENGTH);
    let guard_page_index = (kernel_stack_guard_page_vaddr().into(): usize -
                            kernel_start_vaddr().into(): usize) / BASE_PAGE_LENGTH;

    log!("guard_page_index: {}", guard_page_index);

    for i in 0..kernel_page_size {
        if i % 512 == 0 {
            pd[pd_index(kernel_start_vaddr() + i * BASE_PAGE_LENGTH)] = PDEntry::new(region.start_paddr(), PD_P | PD_RW);
            let npaddr = region.start_paddr() + BASE_PAGE_LENGTH;
            region.move_up(npaddr);
        }

        let pt_entry = pd[pd_index(kernel_start_vaddr() + i * BASE_PAGE_LENGTH)];

        let offset = pt_entry.get_address().into(): usize -
            alloc_base.into(): usize;
        let mut pt_unique = unsafe {
            Unique::new((INITIAL_ALLOC_START_VADDR + offset).into(): usize as *mut PT) };
        
        if i == guard_page_index {
            alloc_kernel_guard_page(unsafe { pt_unique.get_mut() }, i % 512, alloc_base);
        } else {
            alloc_kernel_page(unsafe { pt_unique.get_mut() }, i % 512, alloc_base);
        }
    }
}

// This maps 2MB for allocation region
fn map_alloc_region(alloc_region: &mut MemoryRegion) -> PAddr {
    use arch::paging::{PD_P, PD_RW, PD_PS, PDEntry, flush_all};
    
    let map_alloc_start_vaddr = INITIAL_ALLOC_START_VADDR;
    let map_alloc_pd_index = pd_index(map_alloc_start_vaddr);
    let map_alloc_start_paddr = align_up(alloc_region.start_paddr(), LARGE_PAGE_LENGTH);

    let mut initial_pd = INITIAL_PD.lock();
    initial_pd[map_alloc_pd_index] = PDEntry::new(map_alloc_start_paddr, PD_P | PD_RW | PD_PS);

    // unsafe { super::paging::flush(map_alloc_start_vaddr); }
    unsafe { flush_all(); }

    alloc_region.move_up(map_alloc_start_paddr);
    
    map_alloc_start_paddr
}

pub fn init(mut alloc_region: &mut MemoryRegion) {
    use arch::paging::{switch_to};
    
    let kernel_page_size = block_count(kernel_end_paddr().into(): usize -
                                       kernel_start_paddr().into(): usize, BASE_PAGE_LENGTH);
    let alloc_size = 3 /* PML4, PDPT, and PD, and object pool PT */ +
        block_count(kernel_page_size, 512) /* Kernel page mapping PT */ ;
    
    assert!(alloc_size <= 512);
    assert!(alloc_size * BASE_PAGE_LENGTH < alloc_region.length());
    
    // Before allocation, we need to make sure PAddr + alloc_size is mapped.
    // This is done in the init_pd.
    
    let alloc_base_paddr = map_alloc_region(&mut alloc_region);

    log!("alloc_base_paddr: 0x{:x}", alloc_base_paddr);

    let mut pml4_unique = alloc_kernel_pml4(&mut alloc_region,
                                            alloc_base_paddr);
    let mut pdpt_unique = alloc_kernel_pdpt(&mut alloc_region,
                                            unsafe { pml4_unique.get_mut() },
                                            alloc_base_paddr);
    let mut pd_unique = alloc_kernel_pd(&mut alloc_region,
                                        unsafe { pdpt_unique.get_mut() },
                                        alloc_base_paddr);
    let mut object_pool_pt_unique = alloc_object_pool_pt(&mut alloc_region,
                                                         unsafe { pd_unique.get_mut() },
                                                         alloc_base_paddr);

    alloc_kernel_pts(&mut alloc_region, unsafe { pd_unique.get_mut() }, alloc_base_paddr);
    
    unsafe {
        INITIAL_PD.unbootstrap();
    }
    unsafe { switch_to(KERNEL_PML4.paddr()); }
    unsafe {
        OBJECT_POOL_PT.bootstrap(OBJECT_POOL_PT_VADDR.into(): usize as *mut _);
    }
}
