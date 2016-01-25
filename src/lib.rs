#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(custom_attribute)]
#![feature(asm)]
#![no_std]

extern crate rlibc;
extern crate spin;
#[macro_use] extern crate bitflags;


#[macro_use] mod vga_buffer;
mod multiboot2;
mod memory;

use memory::{AreaFrameAllocator, FrameAllocator};

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    // ATTENTION: we have a very small stack and no guard page

    vga_buffer::clear_screen();

    let boot_info = unsafe { multiboot2::load(multiboot_information_address) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");

    println!("Memory areas:");
    for area in memory_map_tag.memory_areas() {
        println!("    start: 0x{:x}, length: 0x{:x}", area.base_addr, area.length);
    }

    let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size).max().unwrap();

    let multiboot_start = multiboot_information_address;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);

    println!("Kernel start: 0x{:x}, end: 0x{:x}", kernel_start, kernel_end);
    println!("Multiboot start: 0x{:x}, end: 0x{:x}", multiboot_start, multiboot_end);

    println!("Kernel sections:");
    for section in elf_sections_tag.sections() {
        println!("    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
                 section.addr, section.size, section.flags);
    }

    let mut frame_allocator = AreaFrameAllocator::new(
        kernel_start as usize, kernel_end as usize,
        multiboot_start, multiboot_end, memory_map_tag.memory_areas());

    for i in 0.. {
        if let None = frame_allocator.allocate_frame() {
            println!("allocated {} frames", i);
            break;
        }
    }

    println!("Hello, world{}", "!");

    loop{}
}

#[lang = "eh_personality"] extern fn eh_personality() { }
#[lang = "panic_fmt"]
extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    println!("\n\nPANIC in {} at line {}:", file, line);
    println!("    {}", fmt);
    loop{}
}
