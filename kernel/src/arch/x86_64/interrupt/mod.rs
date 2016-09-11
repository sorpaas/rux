mod idt;
mod bit_field;
mod dtables;

use lazy_static;

macro_rules! handler {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("mov rdi, rsp
                      sub rsp, 8 // align the stack pointer
                      call $0"
                      :: "i"($name as extern "C" fn(
                          *const ExceptionStackFrame) -> !)
                      : "rdi" : "intel");
                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    }}
}

macro_rules! handler_with_error_code {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("pop rsi // pop error code into rsi
                      mov rdi, rsp
                      sub rsp, 8 // align the stack pointer
                      call $0"
                      :: "i"($name as extern "C" fn(
                          *const ExceptionStackFrame, u64) -> !)
                      : "rdi","rsi" : "intel");
                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    }}
}

pub struct InterruptInfo {}

lazy_static! {
    pub static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();

        idt.set_handler(0x0, handler!(divide_by_zero_handler));
        idt.set_handler(0x80, handler!(system_call_handler))
            .set_privilege_level(0x3);

        idt
    };
}

#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

extern "C" fn system_call_handler(stack_frame: *const ExceptionStackFrame) -> ! {
    log!("interrupt: system call");
    unsafe {
        let ref exception = *stack_frame;
        log!("instruction pointer: 0x{:x}", exception.instruction_pointer);
        log!("code segment: 0x{:x}", exception.code_segment);
        log!("cpu flags: 0b{:b}", exception.cpu_flags);
        log!("stack pointer: 0x{:x}", exception.stack_pointer);
        log!("stack segment: 0x{:x}", exception.stack_segment);
    }
    loop {}
}

extern "C" fn divide_by_zero_handler(stack_frame: *const ExceptionStackFrame) -> ! {
    log!("interrupt: divide by zero");
    unsafe { log!("{:?}", *stack_frame); }
    loop {}
}

pub fn enable_interrupt() {

}

pub fn disable_interrupt() {

}

pub fn set_interrupt_handler(handler: fn(info: InterruptInfo)) {

}
