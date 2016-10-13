use core::fmt;

pub fn debug(message: &str) {
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

pub struct Writer;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        debug(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! debug {
    ( $($arg:tt)* ) => ({
        // Import the Writer trait (required by write!)
        use core::fmt::Write;
        let _ = write!(&mut $crate::debug::Writer, $($arg)*);
    })
}
