mod uart16550;

use core::panic::PanicInfo;

#[lang="start"]
#[no_mangle]
pub fn kinit(mut hartid: usize, dtb: usize) -> ! {
    let chars = [0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21];

    for c in &chars {
        unsafe { uart16550::putchar(*c); }
    }

    loop { }
}

#[no_mangle]
pub fn trap_handler(regs: *mut *mut usize, mcause: *mut usize, mepc: *mut usize) {
    loop { }
}

#[no_mangle]
pub fn abort() -> ! {
    loop { }
}

#[panic_implementation]
#[no_mangle]
fn kpanic(_: &PanicInfo) -> ! {
    loop { }
}
