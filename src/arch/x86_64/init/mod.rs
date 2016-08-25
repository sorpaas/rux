mod multiboot;
mod info;

use ::{kmain};
pub use self::info::{MemoryRegion, ArchInfo};

/// Kernel entrypoint
#[lang="start"]
#[no_mangle]
pub fn kinit() {
    // let kernel_page_size = block_count(kernel_end_paddr().as_usize() - kernel_start_paddr().as_usize(), BASE_PAGE_LENGTH);

    kmain();
        
    // let bootinfo = unsafe {
    //     multiboot::Multiboot::new(arch::multiboot_address(), |addr, size| {
    //         let ptr = mem::transmute(arch::kernel_internal_to_virtual(addr).as_usize());
    //         Some(slice::from_raw_parts(ptr, size))
    //     })
    // }.unwrap();

    // let mut archinfo = ArchInfo { free_memory_length: 0,
    //                               free_memory_regions: [None; 16] };
    // let mut alloc_region: Option<MemoryRegion> = None;
    
    // for area in bootinfo.memory_regions().unwrap() {
    //     if !(area.memory_type() == RAM) {
    //         continue;
    //     }

    //     let mut cur_region = MemoryRegion {
    //         base_paddr: PAddr::from_u64(area.base_address()),
    //         length: area.length()
    //     };

    //     if cur_region.start_paddr <= kernel_start_paddr() &&
    //         PAddr::from_usize(cur_region.start_paddr.as_usize() + cur_region.length) >= kernel_end_paddr()
    //     {
    //         let npaddr = align_up(kernel_end_paddr(), BASE_PAGE_LENGTH);
    //         cur_region.move_to(npaddr);
    //     }
    // }
}
