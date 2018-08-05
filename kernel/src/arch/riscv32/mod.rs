mod uart16550;

use core::panic::PanicInfo;

pub use self::uart16550::{putchar, getchar};

#[lang="start"]
#[no_mangle]
pub fn kinit(hartid: usize, dtb: usize) -> ! {
    log!("Hello, world!");
    log!("hartid: {}, dtb: {}", hartid, dtb);
    loop {
        let c = unsafe { getchar() };
        if c.is_some() {
            log!("getchar: {:?}", c);
        }
    }
}

#[no_mangle]
pub fn trap_handler(_regs: *mut *mut usize, _mcause: *mut usize, _mepc: *mut usize) {
    log!("A trap happened.");
    loop { }
}

#[no_mangle]
pub fn abort() -> ! {
    loop { }
}

#[panic_implementation]
#[no_mangle]
pub fn kpanic(info: &PanicInfo) -> ! {
    log!("{}", info);
    loop { }
}
