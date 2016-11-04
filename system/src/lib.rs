#![feature(lang_items)]
#![feature(asm)]
#![no_std]

extern crate rlibc;
extern crate abi;

pub use abi::{CapSystemCall, CapSendMessage, TaskBuffer, SystemCallRequest};

pub mod unwind;
pub mod debug;

pub fn system_call(message: CapSystemCall) {
    unsafe {
        let buffer = unsafe { &mut *(0x80001000 as *mut TaskBuffer) };
        let hello = "hello";
        let mut str_buffer: [u8; 32] = [0; 32];
        for (i, u) in hello.as_bytes().iter().enumerate() {
            str_buffer[i] = *u;
        }
        buffer.request = Some(SystemCallRequest::Print(str_buffer, hello.len()));
        system_call_raw()
    }
}

unsafe fn system_call_raw() {
    unsafe {
        asm!("int 80h"
             ::
             : "rax", "rbx", "rcx", "rdx",
               "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"
             : "volatile", "intel");
    }
}
