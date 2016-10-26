mod idt;
mod bit_field;
mod dtables;

use lazy_static;
use cap::{TCBHalf, CapHalf, CPoolHalf, Capability,
          CapReadObject, CapWriteObject, CapSendMessage, CapSystemCall, SystemCallable};
use common::*;

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

        idt.set_handler(0x0, handler!(divide_by_zero_handler));
        idt.set_handler(0x80, handler!(system_call_handler))
            .set_privilege_level(0x3);
        idt.set_handler(0x81, handler!(debug_call_handler))
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

#[derive(Debug, Clone)]
pub struct ThreadRuntime {
    instruction_pointer: u64,
    cpu_flags: u64,
    stack_pointer: u64
}

static mut active_tcb: Option<TCBHalf> = None;
unsafe fn update_active_tcb(stack_frame: &ExceptionStackFrame) {
    let mut tcb = active_tcb.as_mut().unwrap().write();
    let runtime = tcb.runtime_mut();
    runtime.instruction_pointer = stack_frame.instruction_pointer;
    runtime.cpu_flags = stack_frame.cpu_flags;
    runtime.stack_pointer = stack_frame.stack_pointer;
}

impl ThreadRuntime {
    pub unsafe fn switch_to(&self, mut tcb_half: TCBHalf) {
        active_tcb = Some(tcb_half.clone());

        let stack_vaddr = self.stack_pointer as usize;
        let code_start = self.instruction_pointer as usize;
        let cpu_flags = self.cpu_flags as usize;
        let code_seg = 0x28 | 0x3;
        let data_seg = 0x30 | 0x3;

        asm!("mov ds, rax
              mov es, rax
              mov fs, rax
              mov gs, rax

              push rax
              push rbx
              push r8
              push rcx
              push rdx
              iretq"
             :: "{rax}"(data_seg), "{rbx}"(stack_vaddr), "{rcx}"(code_seg), "{rdx}"(code_start), "{r8}"(cpu_flags)
             : "memory" : "intel", "volatile");
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

extern "C" fn system_call_handler(stack_frame: *const ExceptionStackFrame) -> ! {
    log!("interrupt: system call");
    unsafe {
        let ref message = fetch_message!(CapSystemCall);
        log!("message is: {:?}", message);

        let ref exception = *stack_frame;
        update_active_tcb(&exception);

        // let mut tcb = active_tcb.as_mut().unwrap().write();
        // let (target_index, target_cpool_routes) = message.target.split_last().unwrap();
        // let target_cpool = {
        //     match tcb.cpool_mut() {
        //         &mut Capability::CPool(ref mut cpool_half) => {
        //             cpool_half.traverse(target_cpool_routes)
        //         },
        //         _ => None
        //     }
        // };

        // if target_cpool.is_some() {
        //     let mut unwrapped = target_cpool.unwrap();
        //     let mut locked = unwrapped.write();
        //     match locked[*target_index as usize] {
        //         Some(ref mut cap) => {
        //             cap.handle_send(message.message);
        //         },
        //         _ => ()
        //     }
        // }

        // If we didn't call switch_to in the handler, then switch back to the active_tcb.
        active_tcb.as_mut().unwrap().switch_to();
    }
    loop {}
}

extern "C" fn debug_call_handler(stack_frame: *const ExceptionStackFrame) -> ! {
    log!("interrupt: debug call");
    unsafe {
        let ref message = fetch_message!(&str);

        let ref exception = *stack_frame;
        update_active_tcb(&exception);

        log!("[debug] {} from {}", message, unsafe { active_tcb.as_ref().unwrap() });

        active_tcb.as_mut().unwrap().switch_to();
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
