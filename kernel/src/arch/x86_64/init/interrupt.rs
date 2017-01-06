use arch::interrupt::{self, IDT, IO_APIC, LOCAL_APIC, disable_pic};

/// Initialize interrupt. Disable PIC and then initialize APIC
/// together with keyboard interrupt on I/O APIC.
pub fn init() {
    unsafe { disable_pic() };
    IDT.load();

    {
        let mut local_apic = LOCAL_APIC.lock();
        let mut io_apic = IO_APIC.lock();
        let local_apic_id = local_apic.id() as u8;
        let vector_offset = 0x20;
        io_apic.set_irq(0x1, local_apic_id, interrupt::KEYBOARD_INTERRUPT_CODE);

        local_apic.set_siv(0x1FF);
    }
}
