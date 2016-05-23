#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(custom_attribute)]
#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(asm)]
#![feature(step_by)]
#![feature(alloc)]
#![feature(associated_type_defaults)]
#![feature(drop_types_in_const)]
#![no_std]

#[macro_use]
extern crate bitflags;
extern crate rlibc;
extern crate spin;
extern crate x86;
extern crate alloc;

#[macro_use]
mod vga_buffer;
mod multiboot2;
mod common;
mod cap;
mod utils;

use common::*;
use cap::{MemoryBlock, UntypedCapability,
          FrameCapability, GuardedFrameCapability,
          CapabilityPool, CapabilityMove,
          // PageTableCapability
};

use core::mem;
use x86::{controlregs, tlb};

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    enable_nxe_bit();
    enable_write_protect_bit();

    vga_buffer::clear_screen();

    let mut cap_pool = CapabilityPool::new();

    let boot_info = unsafe { multiboot2::load(multiboot_information_address) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");

    println!("Memory areas:");
    for area in memory_map_tag.memory_areas() {
        println!("    start: 0x{:x}, length: 0x{:x}", area.base_addr, area.length);
        cap_pool.put(unsafe { UntypedCapability::bootstrap(area.base_addr as PhysicalAddress, area.length as usize) });
    }

    let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap() as usize;
    let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size - 1).max().unwrap() as usize;

    let multiboot_start = multiboot_information_address as usize;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize) - 1;

    println!("Kernel start: 0x{:x}, end: 0x{:x}", kernel_start, kernel_end);
    println!("Multiboot start: 0x{:x}, end: 0x{:x}", multiboot_start, multiboot_end);

    let mut page_untyped = cap_pool.select(|x: &UntypedCapability| {
        x.block().start_addr() <= kernel_start &&
            x.block().end_addr() >= kernel_end &&
            x.block().start_addr() <= multiboot_start &&
            x.block().end_addr() >= multiboot_end
    }).expect("Unexcepted multiboot memory allocation.");


    let mut kernel_untyped = {
        let target_block_start_addr = page_untyped.block().physical_start_addr();
        UntypedCapability::from_untyped(&mut page_untyped, multiboot_end - target_block_start_addr + 1)
    };

    println!("Page untyped start address: 0x{:x}.", page_untyped.block().start_addr());

    cap_pool.put(page_untyped);
    cap_pool.put(kernel_untyped);

    // let (page_table, ou) = unsafe { PageTableCapability::bootstrap(page_untyped) };
    // println!("Inactive page table capability address: 0x{:x}.", page_table.p4_block().start_addr());
    // page_untyped = ou.expect("Out of memory.");

    // println!("Kernel sections:");
    // for section in elf_sections_tag.sections() {
    //     use multiboot2::ELF_SECTION_ALLOCATED;

    //     println!("    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
    //              section.addr, section.size, section.flags);

    //     if !section.flags().contains(ELF_SECTION_ALLOCATED) {
    //         // section is not loaded to memory
    //         let untyped_start_addr = kernel_untyped.block().start_addr();
    //         assert!(untyped_start_addr <= section.addr as usize);
    //         let (reserved, ou) =
    //             GuardedFrameCapability::from_untyped(kernel_untyped,
    //                                                  (section.size + section.addr) as usize -
    //                                                  untyped_start_addr);
    //         kernel_untyped = ou.expect("Out of memory.");
    //         cap_pool.put(reserved);
    //     } else {
    //         let section_addr = section.addr as usize;
    //         let section_size = section.size as usize;

    //         let flags = EntryFlags::from_elf_section_flags(&section);
    //         let cr3 = unsafe { controlregs::cr3() as usize };

    //         assert!(section_addr % PAGE_SIZE == 0);
    //         let (reserved, ou) =
    //             FrameCapability::from_untyped_fixed(kernel_untyped, section_addr,
    //                                                 utils::necessary_page_count(section_size),
    //                                                 flags);
    //         kernel_untyped = ou.expect("Out of memory.");

    //         // println!("Identity mapping ...");
    //         // let (virt, ou) = page_table.identity_map(reserved, kernel_untyped);
    //         // kernel_untyped = ou.expect("Out of memory.");

    //         cap_pool.put(reserved);
    //     }
    // }

    // {
    //     let (virt, ou) = page_table.identity_map(vga_buffer::WRITER.lock().cap(), page_untyped);
    //     page_untyped = ou.expect("Out of memory.");
    // }

    // let (page_table, old_page_table) = ActivePageTableCapability::switch(page_table, cur_page_table);

    println!("Available slots in kernel's capability pool: {}.", cap_pool.available_count());
    println!("Yeah, the kernel did not crash!");

    loop{}
}

fn stack_overflow(i: usize) {
    stack_overflow(i+1)
}

fn enable_nxe_bit() {
    use x86::msr::{IA32_EFER, rdmsr, wrmsr};

    let nxe_bit = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit);
    }
}

fn enable_write_protect_bit() {
    use x86::controlregs::{cr0, cr0_write};

    let wp_bit = 1 << 16;
    unsafe { cr0_write(cr0() | wp_bit) };
}

#[lang = "eh_personality"] extern fn eh_personality() { }
#[lang = "panic_fmt"]
extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    println!("\n\nPANIC in {} at line {}:", file, line);
    println!("    {}", fmt);
    loop{}
}
