#![feature(asm)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(drop_types_in_const)]
#![feature(unique)]
#![feature(alloc)]

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
mod interrupts;

use common::*;
use cap::{MemoryBlock, UntypedCapability,
          FrameCapability, GuardedCapability,
          CapabilityPool, CapabilityMove,
          PageTableCapability
};

use core::mem;
use x86::{controlregs, tlb};

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    enable_nxe_bit();
    enable_write_protect_bit();

    vga_buffer::clear_screen();
    println!("Size of pool: {}", core::mem::size_of::<CapabilityPool>());

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

    let mut kernel_page_table = PageTableCapability::from_untyped(&mut page_untyped, 0x7e0000000, 512);
    println!("Inactive page table capability address: 0x{:x}.", kernel_page_table.p4_block().start_addr());

    let cr3 = unsafe { controlregs::cr3() as usize };

    println!("Kernel sections:");
    for section in elf_sections_tag.sections() {
        use multiboot2::ELF_SECTION_ALLOCATED;

        println!("    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
                 section.addr, section.size, section.flags);

        let section_size = section.size as usize;
        let section_addr = section.addr as usize;
        let untyped_start_addr = kernel_untyped.block().start_addr();

        assert!(untyped_start_addr <= section_addr);

        if !section.flags().contains(ELF_SECTION_ALLOCATED) {
            // section is not loaded to memory

            let reserved = GuardedCapability::from_untyped_fixed(&mut kernel_untyped,
                                                                 section_addr, section_size);
            cap_pool.put(reserved);
        } else {
            // section loaded to memory

            assert!(section_addr % PAGE_SIZE == 0);
            let flags = EntryFlags::from_elf_section_flags(&section) | USER_ACCESSIBLE;

            let reserved = if cr3 >= section_addr && cr3 < section_addr + section_size {
                assert!(cr3 == section_addr);

                // There will be one guarded page directly on the old P4 table.
                let guarded = GuardedCapability::from_untyped_fixed(&mut kernel_untyped, cr3, PAGE_SIZE);
                cap_pool.put(guarded);

                println!("    (Stack guarded page initialized at: 0x{:x}.)", cr3);

                FrameCapability::from_untyped_fixed(&mut kernel_untyped, cr3 + PAGE_SIZE,
                                                    utils::necessary_page_count(section_size - PAGE_SIZE),
                                                    flags)
            } else {
                FrameCapability::from_untyped_fixed(&mut kernel_untyped, section_addr,
                                                    utils::necessary_page_count(section_size), flags)
            };

            // println!("    (Identity mapping ...)");
            let frame_start_addr = reserved.block().start_addr();
            let frame_count = reserved.count();

            kernel_page_table.create_tables_and_identity_map(reserved, &mut page_untyped);
        }
    }

    unsafe {
        kernel_page_table.create_tables_and_map_vga_buffer(&mut page_untyped);
        kernel_page_table.switch_on();
    }

    println!("Available slots in kernel's capability pool: {}.", cap_pool.available_count());
    println!("Yeah, the kernel did not crash!");

    let test_page_table = PageTableCapability::from_untyped(&mut page_untyped, 0x7e0000000, 512);

    println!("Test page table created successfullly.");

    cap_pool.put(page_untyped);
    cap_pool.put(kernel_untyped);

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

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
