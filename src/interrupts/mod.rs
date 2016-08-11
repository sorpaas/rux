mod idt;
mod bit_field;

use lazy_static;

lazy_static! {
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();

        idt.set_handler(0x0, divide_by_zero_handler);

        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "C" fn divide_by_zero_handler() -> ! {
    use vga_buffer::print_error;

    unsafe { print_error(format_args!("EXCEPTION: DIVIDE BY ZERO")) };
    loop {}
}
