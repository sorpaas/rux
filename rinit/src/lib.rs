#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(unique)]
#![no_std]

#[macro_use]
extern crate system;
extern crate spin;

#[macro_use]
mod vga_buffer;

use core::ops::{Deref};

#[lang="start"]
#[no_mangle]
fn start(_argc: isize, _argv: *const *const u8) {
    system::set_task_buffer(0x80001000);
    let mut v = 0x100;
    system_print!("hello: 0x{:x}", v);
    system::cpool_list_debug();
    for i in 0..0x100 {
        v += 1;
    }
    system_print!("hello: 0x{:x}", v);
    print!("hello: 0x{:x}", v);
    loop {};
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}
