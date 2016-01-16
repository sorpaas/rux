#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(custom_attribute)]
#![feature(asm)]
#![no_std]

extern crate rlibc;
extern crate spin;

#[macro_use]
mod vga_buffer;

#[no_mangle]
pub extern fn rust_main() {
    // ATTENTION: we have a very small stack and no guard page

    vga_buffer::clear_screen();
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
