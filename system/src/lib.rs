#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![no_std]

extern crate rlibc;
extern crate abi;
extern crate spin;

pub mod unwind;
mod call;

pub use self::call::{set_task_buffer, cpool_list_debug, retype_cpool, retype_task,
                     channel_put, channel_take};

use core::fmt;

pub struct PrintWriter {
    buffer: [u8; 32],
    size: usize,
}

impl PrintWriter {
    pub fn new() -> Self {
        PrintWriter {
            buffer: [0u8; 32],
            size: 0,
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
