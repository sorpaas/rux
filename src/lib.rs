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

use common::*;
use cap::{MemoryBlockCapability, PageFrameCapability};
use cap::{CapabilityPool, CapabilityUnion, CapabilityMove};
use cap::UntypedCapability;
use cap::KernelReservedBlockCapability;
use cap::KernelReservedFrameCapability;
use cap::VGABufferCapability;
use cap::paging;
use cap::paging::EntryFlags;
use cap::paging::VirtualAddress;
use cap::paging::{InactivePageTableCapability, ActivePageTableCapability};

use core::mem;
use x86::{controlregs, tlb};

struct Target(UntypedCapability);

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    use cap::paging::entry::WRITABLE;

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

        cap_pool.put(unsafe { UntypedCapability::new(area.base_addr as usize, area.length as usize) });
    }

    let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap() as usize;
    let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size - 1).max().unwrap() as usize;

    let multiboot_start = multiboot_information_address as usize;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize) - 1;

    println!("Kernel start: 0x{:x}, end: 0x{:x}", kernel_start, kernel_end);
    println!("Multiboot start: 0x{:x}, end: 0x{:x}", multiboot_start, multiboot_end);

    let target_untyped: UntypedCapability = cap_pool.select(|x: &UntypedCapability| {
        x.block_start_addr() <= kernel_start &&
            x.block_end_addr() >= kernel_end &&
            x.block_start_addr() <= multiboot_start &&
            x.block_end_addr() >= multiboot_end
    }).expect("Illegal memory allocation.");
    let target_block_start_addr = target_untyped.block_start_addr();
    let (otkernel, otpage) = UntypedCapability::from_untyped(target_untyped, multiboot_end - target_block_start_addr + 1);

    let mut kernel_untyped = otkernel;
    let mut page_untyped = otpage.expect("Out of memory");

    println!("Page untyped start address: 0x{:x}.", page_untyped.block_start_addr());

    let cr3 = unsafe { controlregs::cr3() } as usize;
    let cur_page_table = unsafe { ActivePageTableCapability::bootstrap() };
    println!("Cr3 value: 0x{:x}.", unsafe { controlregs::cr3() } as usize);
    let (page_table, ou) = InactivePageTableCapability::from_untyped(page_untyped, &cur_page_table);
    let page_table = page_table.expect("Initialize page table failed.");
    println!("Inactive page table capability address: 0x{:x}.", (&page_table as *const _) as usize);
    page_untyped = ou.expect("Out of memory.");

    println!("Kernel sections:");
    for section in elf_sections_tag.sections() {
        use multiboot2::ELF_SECTION_ALLOCATED;

        println!("    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
                 section.addr, section.size, section.flags);

        if !section.flags().contains(ELF_SECTION_ALLOCATED) {
            // section is not loaded to memory
            let (reserved, ou) = KernelReservedBlockCapability::from_untyped(kernel_untyped, section.addr as usize, section.size as usize);
            kernel_untyped = ou.expect("Out of memory.");
            cap_pool.put(reserved.expect("Reserved should be allocated."));
        } else {
            let flags = EntryFlags::from_elf_section_flags(&section);
            let guarded_frame = if cr3 >= section.addr as usize &&
                cr3 < section.addr as usize + (cap::utils::necessary_page_count(section.size as usize) - 1) * PAGE_SIZE
                { Some(cr3) } else { None };
            let (reserved, ou) = KernelReservedFrameCapability::from_untyped(kernel_untyped, section.addr as usize, section.size as usize, guarded_frame, flags);
            let reserved = reserved.expect("Reserved should be allocated.");
            kernel_untyped = ou.expect("Out of memory.");

            let (virt, ou) = page_table.identity_map(&reserved, page_untyped);
            page_untyped = ou.expect("Out of memory.");

            // println!("New untyped start address: 0x{:x}.", target_untyped.block_start_addr());
            // println!("New untyped size: 0x{:x}.", target_untyped.block_size());


            // println!("Reserved frame start address: 0x{:x}.", reserved.frame_start_addr());
            // println!("Reserved frame size: 0x{:x}.", reserved.frame_size());

            cap_pool.put(reserved);
        }
    }

    {
        let (virt, ou) = page_table.identity_map(vga_buffer::WRITER.lock().cap(), page_untyped);
        page_untyped = ou.expect("Out of memory.");
    }

    let (page_table, old_page_table) = ActivePageTableCapability::switch(page_table, cur_page_table);

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
