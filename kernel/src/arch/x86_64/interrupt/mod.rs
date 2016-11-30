mod idt;
mod bit_field;
mod dtables;
mod apic;

#[macro_use]
mod switch;

use lazy_static;
use common::*;
use self::switch::{last_exception_return_value, switch_to_raw, Exception};

pub use self::switch::{HandlerFunc};
pub use self::apic::{LOCAL_APIC, IO_APIC};

macro_rules! fetch_message {
    ($t: ty) => {
        *({
            let param: u64;
            asm!("":"={r15}"(param));

            param
        } as *const $t)
    }
}

pub struct InterruptInfo {}

lazy_static! {
    pub static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();

        // idt.set_handler(0x0, handler!(divide_by_zero_handler));
        idt.set_handler(0x80, switch::system_call_return_to_raw)
            .set_privilege_level(0x3);
        idt.set_handler(0x81, switch::debug_call_return_to_raw)
            .set_privilege_level(0x3);
        idt.set_handler(0x41, switch::keyboard_return_to_raw)
            .set_privilege_level(0x3);

        idt
    };
}

#[derive(Debug)]
pub struct TaskRuntime {
    instruction_pointer: u64,
    cpu_flags: u64,
    stack_pointer: u64
}

impl Default for TaskRuntime {
    fn default() -> TaskRuntime {
        TaskRuntime {
            instruction_pointer: 0x0,
            cpu_flags: 0b110,
            stack_pointer: 0x0
        }
    }
}

impl TaskRuntime {
    pub unsafe fn switch_to(&mut self) -> (u64, Option<u64>) {
        switch_to_raw(self.stack_pointer, self.instruction_pointer, self.cpu_flags);

        let exception = last_exception_return_value().unwrap();

        self.instruction_pointer = exception.instruction_pointer;
        self.cpu_flags = exception.cpu_flags;
        self.stack_pointer = exception.stack_pointer;

        return (exception.exception_code, exception.error_code);
    }

    pub fn set_instruction_pointer(&mut self, instruction_pointer: VAddr) {
        self.instruction_pointer = instruction_pointer.into();
    }

    pub fn set_stack_pointer(&mut self, stack_pointer: VAddr) {
        self.stack_pointer = stack_pointer.into();
    }
}

pub unsafe fn enable_interrupt() { }
pub unsafe fn disable_interrupt() { }
pub unsafe fn set_interrupt_handler() { }
