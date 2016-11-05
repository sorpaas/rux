#![feature(lang_items)]
#![feature(asm)]
#![no_std]

#[macro_use]
extern crate system;

use core::ops::{Deref};

struct A;
struct B(A);

#[lang="start"]
#[no_mangle]
fn start(_argc: isize, _argv: *const *const u8) {
    system::set_task_buffer(0x80001000);
    system_print!("hello: {}", "a value");
    loop {};
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}
