#![feature(lang_items)]
#![feature(asm)]
#![no_std]

extern crate rlibc;
extern crate abi;

pub use abi::{CapSystemCall, CapSendMessage};

pub mod unwind;
pub mod debug;

pub fn system_call(message: CapSystemCall) {
    unsafe {
        system_call_raw(&message as *const CapSystemCall as u64)
    }
}

unsafe fn system_call_raw(param: u64) {
    let p = param;
    unsafe {
        asm!("int 80h"
             :: "{r15}"(p)
             : "rax", "rbx", "rcx", "rdx",
               "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"
             : "volatile", "intel");
    }
}
