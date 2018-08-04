use core::panic::PanicInfo;

#[lang="start"]
#[no_mangle]
pub fn kinit() {
    loop { }
}

#[panic_implementation]
fn kpanic(_: &PanicInfo) -> ! {
    loop { }
}
