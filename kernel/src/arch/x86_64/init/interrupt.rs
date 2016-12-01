use arch::interrupt::{IDT, IO_APIC, LOCAL_APIC, disable_pic};

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
