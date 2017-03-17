#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(alloc)]
#![no_std]

#[macro_use]
extern crate system;
extern crate spin;
extern crate selfalloc;
extern crate alloc;

use system::{CAddr};

#[lang="start"]
#[no_mangle]
fn start(_argc: isize, _argv: *const *const u8) {
    unsafe { system::set_task_buffer_addr(0x90001000); }
    unsafe { selfalloc::setup_allocator(CAddr::from(2), CAddr::from(3), 0x1000000000); }

    // Test allocator
    {
        use alloc::boxed::Box;
        use core::ops::Deref;
        let heap_test = Box::new(42);
        if heap_test.deref() != &42 {
            system::debug_test_fail();
        }
    }

    system::debug_test_succeed();
}
