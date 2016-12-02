mod idt;
mod bit_field;
mod dtables;
mod apic;
mod pic;

#[macro_use]
mod switch;

use lazy_static;
use common::*;
use self::switch::{last_exception_return_value, switch_to_raw, ExceptionInfo};

pub use self::switch::{HandlerFunc};
pub use self::apic::{LOCAL_APIC, IO_APIC};
pub use self::pic::{disable_pic};

pub type InterruptVector = u64;

pub const TIMER_INTERRUPT_CODE: InterruptVector = 0x40;
pub const SPURIOUS_INTERRUPT_CODE: InterruptVector = 0xFF;
pub const KEYBOARD_INTERRUPT_CODE: InterruptVector = 0x21;
pub const SYSTEM_CALL_INTERRUPT_CODE: InterruptVector = 0x80;
pub const DEBUG_CALL_INTERRUPT_CODE: InterruptVector = 0x81;

return_to_raw_fn!(timer_return_to_raw, TIMER_INTERRUPT_CODE);
return_to_raw_fn!(spurious_return_to_raw, SPURIOUS_INTERRUPT_CODE);
return_to_raw_fn!(keyboard_return_to_raw, KEYBOARD_INTERRUPT_CODE);
return_to_raw_fn!(system_call_return_to_raw, SYSTEM_CALL_INTERRUPT_CODE);
return_to_raw_fn!(debug_call_return_to_raw, DEBUG_CALL_INTERRUPT_CODE);

lazy_static! {
    pub static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();

        idt.set_handler(SYSTEM_CALL_INTERRUPT_CODE, system_call_return_to_raw)
            .set_privilege_level(0x3);
        idt.set_handler(DEBUG_CALL_INTERRUPT_CODE, debug_call_return_to_raw)
            .set_privilege_level(0x3);
        idt.set_handler(KEYBOARD_INTERRUPT_CODE, keyboard_return_to_raw)
            .set_privilege_level(0x3);
        idt.set_handler(SPURIOUS_INTERRUPT_CODE, spurious_return_to_raw)
            .set_privilege_level(0x3);
        idt.set_handler(TIMER_INTERRUPT_CODE, timer_return_to_raw)
            .set_privilege_level(0x3);

        idt
    };
}

#[derive(Debug)]
pub enum Exception {
    SystemCall,
    DebugCall,
    Keyboard,
    Spurious,
    Timer
}

impl Exception {
    fn new(code: u64, error: Option<u64>) -> Exception {
        match code {
            TIMER_INTERRUPT_CODE => Exception::Timer,
            SPURIOUS_INTERRUPT_CODE => Exception::Spurious,
            KEYBOARD_INTERRUPT_CODE => Exception::Keyboard,
            SYSTEM_CALL_INTERRUPT_CODE => Exception::SystemCall,
            DEBUG_CALL_INTERRUPT_CODE => Exception::DebugCall,
            _ => panic!(),
        }
    }

    pub unsafe fn send_eoi(&self) {
        match self {
            &Exception::Timer => LOCAL_APIC.lock().eoi(),
            &Exception::Keyboard => LOCAL_APIC.lock().eoi(),
            _ => (),
        }
    }
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
            cpu_flags: 0b11001000000110,
            stack_pointer: 0x0
        }
    }
}

impl TaskRuntime {
    pub unsafe fn switch_to(&mut self, mode_change: bool) -> Exception {
        let code_seg: u64 = if mode_change { 0x28 | 0x3 } else { 0x8 | 0x0 };
        let data_seg: u64 = if mode_change { 0x30 | 0x3 } else { 0x10 | 0x0 };

        switch_to_raw(self.stack_pointer, self.instruction_pointer, self.cpu_flags, code_seg, data_seg);

        let exception_info = last_exception_return_value().unwrap();

        self.instruction_pointer = exception_info.instruction_pointer;
        self.cpu_flags = exception_info.cpu_flags;
        self.stack_pointer = exception_info.stack_pointer;

        let exception = Exception::new(exception_info.exception_code, exception_info.error_code);
        unsafe { exception.send_eoi(); }

        return exception;
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
