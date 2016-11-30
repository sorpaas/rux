use arch::interrupt::{IDT};
use arch::{inportb, outportb};

unsafe fn disable_pic() {
    // Set ICW1
    outportb(0x20, 0x11);
    outportb(0xa0, 0x11);

    // Set IWC2 (IRQ base offsets)
    outportb(0x21, 0xe0);
    outportb(0xa1, 0xe8);

    // Set ICW3
    outportb(0x21, 4);
    outportb(0xa1, 2);

    // Set ICW4
    outportb(0x21, 1);
    outportb(0xa1, 1);

    // Set OCW1 (interrupt masks)
    outportb(0x21, 0xff);
    outportb(0xa1, 0xff);
}

pub fn init() {
    IDT.load();
    unsafe { disable_pic() };
}
