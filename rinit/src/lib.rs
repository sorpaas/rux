#![feature(lang_items)]
#![feature(asm)]
#![no_std]

#[macro_use]
extern crate system;

use system::debug;

#[lang="start"]
#[no_mangle]
fn start(_argc: isize, _argv: *const *const u8) {
    // divide_by_zero();
    debug!("Test 1");
    debug!("Test 2");
    loop {};
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}
