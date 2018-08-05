mod uart16550;

use core::panic::PanicInfo;

#[lang="start"]
#[no_mangle]
#[naked]
pub unsafe fn kreset() -> ! {
    asm!("
      li x1, 0
      li x2, 0
      li x3, 0
      li x4, 0
      li x5, 0
      li x6, 0
      li x7, 0
      li x8, 0
      li x9, 0
      // save a0 and a1; arguments from previous boot loader stage:
      // li x10, 0
      // li x11, 0
      li x12, 0
      li x13, 0
      li x14, 0
      li x15, 0
      li x16, 0
      li x17, 0
      li x18, 0
      li x19, 0
      li x20, 0
      li x21, 0
      li x22, 0
      li x23, 0
      li x24, 0
      li x25, 0
      li x26, 0
      li x27, 0
      li x28, 0
      li x29, 0
      li x30, 0
      li x31, 0
      tail kinit");
    ::core::intrinsics::unreachable()
}

#[no_mangle]
pub fn kinit(mut hartid: usize, dtb: usize) -> ! {
    unsafe { uart16550::putchar(0x48); }
    unsafe { uart16550::putchar(0x65); }
    unsafe { uart16550::putchar(0x6C); }
    unsafe { uart16550::putchar(0x6C); }
    unsafe { uart16550::putchar(0x6F); }

    // let chars = [0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21];

    // for c in &chars {
    //     unsafe { uart16550::putchar(*c); }
    // }

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
