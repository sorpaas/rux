mod uart16550;

use core::panic::PanicInfo;

pub use self::uart16550::{putchar, getchar};

#[lang="start"]
#[no_mangle]
pub fn kinit(_hartid: usize, _dtb: usize) -> ! {
    log!("Hello, world!");

    loop { }
}

#[no_mangle]
pub fn trap_handler(_regs: *mut *mut usize, _mcause: *mut usize, _mepc: *mut usize) {
    loop { }
}

#[no_mangle]
pub fn abort() -> ! {
    loop { }
}

#[panic_implementation]
#[no_mangle]
pub fn kpanic(_: &PanicInfo) -> ! {
    loop { }
}
