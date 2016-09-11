#![feature(lang_items)]
#![feature(asm)]
#![no_std]

extern crate rlibc;

mod unwind;

#[lang="start"]
#[no_mangle]
fn start(_argc: isize, _argv: *const *const u8) {
    // divide_by_zero();
    unsafe { asm!("int 80h" :::: "volatile", "intel"); }
    loop {};
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}
