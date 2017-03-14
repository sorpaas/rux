#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![no_std]

extern crate rlibc;
extern crate abi;
extern crate spin;

pub mod unwind;
mod call;

pub use self::call::{cpool_list_debug, retype_cpool, retype_task,
                     channel_put, channel_take,
                     channel_put_raw, channel_take_raw,
                     channel_put_cap, channel_take_cap,
                     task_set_stack_pointer, task_set_instruction_pointer,
                     task_set_cpool, task_set_top_page_table, task_set_buffer,
                     task_set_active, task_set_inactive};
pub use abi::{CAddr, ChannelMessage};

use core::fmt;

const STACK_LENGTH: usize = 4 * 4096;
pub fn task_buffer_loc() -> usize {
    // We create a random value on stack, lookup its address, and go
    // to the top of the stack possible as the kernel buffer address
    // storage.

    let mut v: usize = 0xdeadbeaf;
    let v_addr = &mut v as *mut usize as usize;

    return (v_addr - (v_addr % STACK_LENGTH));
}

pub fn task_buffer_addr() -> usize {
    unsafe {
        let loc = task_buffer_loc() as *mut usize;
        return *loc;
    }
}

pub unsafe fn set_task_buffer_addr(addr: usize) {
    use core::ptr::write;

    let loc = task_buffer_loc() as *mut usize;
    write(loc, addr);
}

pub struct PrintWriter {
    buffer: [u8; 32],
    size: usize
}

impl PrintWriter {
    pub fn new() -> Self {
        PrintWriter {
            buffer: [0u8; 32],
            size: 0
        }
    }

    pub fn flush(&mut self) {
        if self.size > 0 {
            call::print(self.buffer.clone(), self.size);
            self.buffer = [0u8; 32];
            self.size = 0;
        }
    }
}

impl fmt::Write for PrintWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for u in s.as_bytes().iter() {
            self.buffer[self.size] = *u;
            self.size += 1;

            if self.size >= 32 {
                self.flush();
            }
        }
        Result::Ok(())
    }
}

impl Drop for PrintWriter {
    fn drop(&mut self) {
        self.flush();
    }
}

#[macro_export]
macro_rules! system_print {
    ( $($arg:tt)* ) => ({
        use core::fmt::Write;
        let _ = write!(&mut $crate::PrintWriter::new(), $($arg)*);
    })
}
