use arch::interrupt::{IDT, IO_APIC, LOCAL_APIC};
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
    unsafe { disable_pic() };

    let local_apic_id = LOCAL_APIC.lock().id() as u8;
    let vector_offset = 0x40;
    let mut io_apic = IO_APIC.lock();
    for i in 0..24 {
        io_apic.set_irq(i as u8, local_apic_id, 0x40 + i as u8);
    }

    log!("SIV is: 0x{:x}", LOCAL_APIC.lock().siv());

    IDT.load();
}
