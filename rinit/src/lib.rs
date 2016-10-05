#![feature(lang_items)]
#![feature(asm)]
#![no_std]

extern crate system;

#[lang="start"]
#[no_mangle]
fn start(_argc: isize, _argv: *const *const u8) {
    // divide_by_zero();
    unsafe {
        debug("Test 1");
        debug("Test 2");
    }
    loop {};
}

fn debug(message: &str) {
    unsafe {
        debug_raw(&message as *const &str as u64)
    }
}

unsafe fn debug_raw(param: u64) {
    let p = param;
    unsafe {
        asm!("int 81h"
             :: "{r15}"(p)
             : "rax", "rbx", "rcx", "rdx",
               "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"
             : "volatile", "intel");
    }
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}
