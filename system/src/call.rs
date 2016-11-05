use abi::{SystemCall, TaskBuffer};

pub fn print(buffer: [u8; 32], size: usize) {
    let result = system_call(SystemCall::Print {
        request: (buffer, size)
    });
}

pub fn cpool_list_debug() {
    system_call(SystemCall::CPoolListDebug);
}

static mut TASK_BUFFER_ADDR: Option<usize> = None;

pub fn set_task_buffer(addr: usize) {
    unsafe { TASK_BUFFER_ADDR = Some(addr); }
}

fn system_call(message: SystemCall) -> SystemCall {
    unsafe {
        let buffer = unsafe { &mut *(TASK_BUFFER_ADDR.unwrap() as *mut TaskBuffer) };
        buffer.call = Some(message);
        system_call_raw();
        buffer.call.take().unwrap()
    }
}

unsafe fn system_call_raw() {
    unsafe {
        asm!("int 80h"
             ::
             : "rax", "rbx", "rcx", "rdx",
               "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"
             : "volatile", "intel");
    }
}
