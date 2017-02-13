use abi::{SystemCall, TaskBuffer, CAddr, ChannelMessage};
use spin::{Mutex};

pub fn retype_cpool(addr: usize, source: CAddr, target: CAddr) {
    system_call(SystemCall::RetypeCPool {
        request: (source, target),
    }, addr);
}

pub fn retype_task(addr: usize, source: CAddr, target: CAddr) {
    system_call(SystemCall::RetypeTask {
        request: (source, target),
    }, addr);
}

pub fn task_set_instruction_pointer(addr: usize, target: CAddr, ptr: u64) {
    system_call(SystemCall::TaskSetInstructionPointer {
        request: (target, ptr),
    }, addr);
}

pub fn task_set_stack_pointer(addr: usize, target: CAddr, ptr: u64) {
    system_call(SystemCall::TaskSetStackPointer {
        request: (target, ptr),
    }, addr);
}

pub fn task_set_cpool(addr: usize, target: CAddr, cpool: CAddr) {
    system_call(SystemCall::TaskSetCPool {
        request: (target, cpool),
    }, addr);
}

pub fn task_set_top_page_table(addr: usize, target: CAddr, table: CAddr) {
    system_call(SystemCall::TaskSetTopPageTable {
        request: (target, table),
    }, addr);
}

pub fn task_set_buffer(addr: usize, target: CAddr, buffer: CAddr) {
    system_call(SystemCall::TaskSetBuffer {
        request: (target, buffer),
    }, addr);
}

pub fn task_set_active(addr: usize, target: CAddr) {
    system_call(SystemCall::TaskSetActive {
        request: target
    }, addr);
}

pub fn task_set_inactive(addr: usize, target: CAddr) {
    system_call(SystemCall::TaskSetInactive {
        request: target
    }, addr);
}

pub fn channel_take(addr: usize, target: CAddr) -> ChannelMessage {
    let result = system_call(SystemCall::ChannelTake {
        request: target,
        response: None
    }, addr);
    match result {
        SystemCall::ChannelTake {
            request: _,
            response: response,
        } => {
            return response.unwrap()
        },
        _ => panic!(),
    };
}

pub fn channel_put(addr: usize, target: CAddr, value: ChannelMessage) {
    system_call(SystemCall::ChannelPut {
        request: (target, value)
    }, addr);
}

pub fn print(addr: usize, buffer: [u8; 32], size: usize) {
    let result = system_call(SystemCall::Print {
        request: (buffer, size)
    }, addr);
}

pub fn cpool_list_debug(addr: usize) {
    system_call(SystemCall::CPoolListDebug, addr);
}

fn system_call(message: SystemCall, addr: usize) -> SystemCall {
    unsafe {
        let buffer = unsafe { &mut *(addr as *mut TaskBuffer) };
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
