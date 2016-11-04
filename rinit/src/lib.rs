#![feature(lang_items)]
#![feature(asm)]
#![no_std]

#[macro_use]
extern crate system;

use system::{debug, system_call, CapSystemCall, CapSendMessage};
use core::ops::{Deref};

struct A;
struct B(A);

impl Deref for B {
    type Target = A;

    fn deref(&self) -> &A { &self.0 }
}

impl A {
    pub fn a(&self) {
        debug!("a called!");
    }
}

impl B {
    pub fn b(&self) {
        debug!("b called!");
    }
}

#[lang="start"]
#[no_mangle]
fn start(_argc: isize, _argv: *const *const u8) {
    // divide_by_zero();
    // let b = B(A);
    // b.a();
    // b.b();
    // debug!("Test 1");
    // debug!("Test 2");
    system_call(CapSystemCall {
        target: &[0, 0, 20],
        message: CapSendMessage::TCBYield
    });
    system_call(CapSystemCall {
        target: &[0, 0, 20],
        message: CapSendMessage::TCBYield
    });
    system_call(CapSystemCall {
        target: &[0, 0, 20],
        message: CapSendMessage::TCBYield
    });
    loop {};
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}
