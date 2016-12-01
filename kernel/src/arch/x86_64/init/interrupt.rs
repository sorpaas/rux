use arch::interrupt::{IDT, IO_APIC, LOCAL_APIC, disable_pic};

pub fn init() {
    unsafe { disable_pic() };
    IDT.load();

    let mut local_apic = LOCAL_APIC.lock();
    let mut io_apic = IO_APIC.lock();
    let local_apic_id = local_apic.id() as u8;
    let vector_offset = 0x20;
    for i in 0..16 {
        io_apic.set_irq(i as u8, local_apic_id, vector_offset + i as u8);
    }

    local_apic.set_siv(0x1FF);
}
