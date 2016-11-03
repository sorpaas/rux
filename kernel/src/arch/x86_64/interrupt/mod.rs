mod idt;
mod bit_field;
mod dtables;

#[macro_use]
mod switch;

use lazy_static;
use common::*;
use self::switch::{last_exception_return_value, switch_to_raw, Exception};

pub use self::switch::{HandlerFunc};

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

        idt
    };
}

#[derive(Debug)]
pub struct ThreadRuntime {
    instruction_pointer: u64,
    cpu_flags: u64,
    stack_pointer: u64
}

impl ThreadRuntime {
    pub unsafe fn switch_to(&mut self) -> (u64, Option<u64>) {
        switch_to_raw(self.stack_pointer, self.instruction_pointer, self.cpu_flags);

        let exception = last_exception_return_value().unwrap();
        log!("exception = {:?}", exception);

        self.instruction_pointer = exception.instruction_pointer;
        self.cpu_flags = exception.cpu_flags;
        self.stack_pointer = exception.stack_pointer;

        return (exception.exception_code, exception.error_code);
    }

    pub fn new(instruction_pointer: VAddr, cpu_flags: u64, stack_pointer: VAddr) -> ThreadRuntime {
        ThreadRuntime {
            instruction_pointer: instruction_pointer.into(),
            cpu_flags: cpu_flags,
            stack_pointer: stack_pointer.into()
        }
    }

    pub fn set_instruction_pointer(&mut self, instruction_pointer: VAddr) {
        self.instruction_pointer = instruction_pointer.into();
    }

    pub fn set_stack_pointer(&mut self, stack_pointer: VAddr) {
        self.stack_pointer = stack_pointer.into();
    }
}

// extern "C" fn system_call_handler(stack_frame: *const ExceptionStackFrame) {
//     log!("interrupt: system call");
//     unsafe {
//         let ref message = fetch_message!(CapSystemCall);
//         log!("message is: {:?}", message);

//         let ref exception = *stack_frame;
//         update_active_tcb(&exception);

//         let mut tcb = active_tcb.as_mut().unwrap().write();
//         let (target_index, target_cpool_routes) = message.target.split_last().unwrap();
//         let target_cpool = {
//             match tcb.cpool_mut() {
//                 &mut Capability::CPool(ref mut cpool_half) => {
//                     cpool_half.traverse(target_cpool_routes)
//                 },
//                 _ => None
//             }
//         };

//         if target_cpool.is_some() {
//             let mut unwrapped = target_cpool.unwrap();
//             let mut locked = unwrapped.write();
//             match locked[*target_index as usize] {
//                 Some(ref mut cap) => {
//                     cap.handle_send(message.message);
//                 },
//                 _ => ()
//             }
//         }

//         // If we didn't call switch_to in the handler, then switch back to the active_tcb.
//         active_tcb.as_mut().unwrap().switch_to();
//     }
//     loop {}
// }

// extern "C" fn debug_call_handler(stack_frame: *const ExceptionStackFrame) {
//     log!("interrupt: debug call");
//     unsafe {
//         let ref message = fetch_message!(&str);

//         let ref exception = *stack_frame;
//         update_active_tcb(&exception);

//         log!("[debug] {} from {}", message, unsafe { active_tcb.as_ref().unwrap() });

//         // active_tcb.as_mut().unwrap().switch_to();
//     }
//     // loop {}
// }

// extern "C" fn divide_by_zero_handler(stack_frame: *const ExceptionStackFrame) {
//     log!("interrupt: divide by zero");
//     unsafe { log!("{:?}", *stack_frame); }
//     loop {}
// }

pub unsafe fn enable_interrupt() { }
pub unsafe fn disable_interrupt() { }
pub unsafe fn set_interrupt_handler() { }
