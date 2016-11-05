use abi::{SystemCall, TaskBuffer};
use spin::{Mutex};

pub fn print(buffer: [u8; 32], size: usize) {
    let result = system_call(SystemCall::Print {
        request: (buffer, size)
    });
}

pub fn cpool_list_debug() {
    system_call(SystemCall::CPoolListDebug);
}

static TASK_BUFFER_ADDR: Mutex<Option<usize>> = Mutex::new(None);

pub fn set_task_buffer(addr: usize) {
    *TASK_BUFFER_ADDR.lock() = Some(addr);
}

fn system_call(message: SystemCall) -> SystemCall {
    unsafe {
        let buffer = unsafe { &mut *(TASK_BUFFER_ADDR.lock().unwrap() as *mut TaskBuffer) };
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
