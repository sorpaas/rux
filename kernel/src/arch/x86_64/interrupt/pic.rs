use arch::{inportb, outportb, io_wait};

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const PIC_EOI: u8 = 0x20;
const ICW1_ICW4: u8 = 0x01;
const ICW1_SINGLE: u8 = 0x02;
const ICW1_INTERVAL4: u8 = 0x04;
const ICW1_LEVEL: u8 = 0x08;
const ICW1_INIT: u8 = 0x10;
const ICW4_8086: u8 = 0x01;
const ICW4_AUTO: u8 = 0x02;
const ICW4_BUF_SLAVE: u8 = 0x08;
const ICW4_BUF_MASTER: u8 = 0x0C;
const ICW4_SFNM: u8 = 0x10;

pub unsafe fn disable_pic() {
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

pub unsafe fn send_pic_eoi(irq: u8) {
    if (irq >= 8) {
        outportb(PIC2_COMMAND, PIC_EOI);
    } else {
        outportb(PIC1_COMMAND, PIC_EOI);
    }
}

pub unsafe fn enable_pic(master_offset: u8, slave_offset: u8) {
    let a1 = inportb(PIC1_DATA);
    let a2 = inportb(PIC2_DATA);

    outportb(PIC1_COMMAND, ICW1_INIT + ICW1_ICW4);
    outportb(PIC2_COMMAND, ICW1_INIT + ICW1_ICW4);
    outportb(PIC1_DATA, master_offset);
    outportb(PIC2_DATA, slave_offset);
    outportb(PIC1_DATA, 4);
    outportb(PIC2_DATA, 2);
    outportb(PIC1_DATA, ICW4_8086);
    outportb(PIC2_DATA, ICW4_8086);
    outportb(PIC1_DATA, 0x0);
    outportb(PIC2_DATA, 0x0);

    outportb(PIC1_COMMAND, 0x20);
    outportb(PIC2_COMMAND, 0x20);
}
