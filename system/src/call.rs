use abi::{SystemCall, TaskBuffer, CAddr, ChannelMessage};
use spin::{Mutex};
use core::any::{TypeId, Any};
use super::{task_buffer_addr};

pub fn retype_raw_page_free(source: CAddr) -> CAddr {
    let result = system_call(SystemCall::RetypeRawPageFree {
        request: source,
        response: None
    });
    match result {
        SystemCall::RetypeRawPageFree {
            request: request,
            response: response,
        } => { return response.unwrap(); },
        _ => panic!(),
    };
}

pub fn map_raw_page_free(untyped: CAddr, pt: CAddr, page: CAddr, addr: usize) {
    system_call(SystemCall::MapRawPageFree {
        untyped: untyped,
        request: (pt, page, addr),
    });
}

pub fn retype_cpool(source: CAddr, target: CAddr) {
    system_call(SystemCall::RetypeCPool {
        request: (source, target),
    });
}

pub fn retype_task(source: CAddr, target: CAddr) {
    system_call(SystemCall::RetypeTask {
        request: (source, target),
    });
}

pub fn task_set_instruction_pointer(target: CAddr, ptr: u64) {
    system_call(SystemCall::TaskSetInstructionPointer {
        request: (target, ptr),
    });
}

pub fn task_set_stack_pointer(target: CAddr, ptr: u64) {
    system_call(SystemCall::TaskSetStackPointer {
        request: (target, ptr),
    });
}

pub fn task_set_cpool(target: CAddr, cpool: CAddr) {
    system_call(SystemCall::TaskSetCPool {
        request: (target, cpool),
    });
}

pub fn task_set_top_page_table(target: CAddr, table: CAddr) {
    system_call(SystemCall::TaskSetTopPageTable {
        request: (target, table),
    });
}

pub fn task_set_buffer(target: CAddr, buffer: CAddr) {
    system_call(SystemCall::TaskSetBuffer {
        request: (target, buffer),
    });
}

pub fn task_set_active(target: CAddr) {
    system_call(SystemCall::TaskSetActive {
        request: target
    });
}

pub fn task_set_inactive(target: CAddr) {
    system_call(SystemCall::TaskSetInactive {
        request: target
    });
}

fn channel_take_nonpayload(target: CAddr) -> ChannelMessage {
    let result = system_call(SystemCall::ChannelTake {
        request: target,
        response: None
    });
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

pub fn channel_take_raw(target: CAddr) -> u64 {
    let result = channel_take_nonpayload(target);
    match result {
        ChannelMessage::Raw(v) => return v,
        _ => panic!(),
    };
}

pub fn channel_take_cap(target: CAddr) -> CAddr {
    let result = channel_take_nonpayload(target);
    match result {
        ChannelMessage::Cap(v) => return v.unwrap(),
        _ => panic!(),
    };
}

pub fn channel_take<T: Any + Clone>(target: CAddr) -> T {
    let (result, payload) = system_call_take_payload(SystemCall::ChannelTake {
        request: target,
        response: None
    });
    match result {
        SystemCall::ChannelTake {
            request: _,
            response: Some(ChannelMessage::Payload),
        } => {
            return payload;
        },
        _ => panic!(),
    };
}

pub fn channel_put_raw(target: CAddr, value: u64) {
    system_call(SystemCall::ChannelPut {
        request: (target, ChannelMessage::Raw(value))
    });
}

pub fn channel_put_cap(target: CAddr, value: CAddr) {
    system_call(SystemCall::ChannelPut {
        request: (target, ChannelMessage::Cap(Some(value)))
    });
}

pub fn channel_put<T: Any + Clone>(target: CAddr, value: T) {
    system_call_put_payload(SystemCall::ChannelPut {
        request: (target, ChannelMessage::Payload)
    }, value);
}

pub fn print(buffer: [u8; 32], size: usize) {
    let result = system_call(SystemCall::Print {
        request: (buffer, size)
    });
}

#[cfg(feature="kernel_debug")]
pub fn debug_cpool_list() {
    system_call(SystemCall::DebugCPoolList);
}

#[cfg(feature="kernel_debug")]
pub fn debug_test_succeed() {
    system_call(SystemCall::DebugTestSucceed);
    loop {}
}

#[cfg(feature="kernel_debug")]
pub fn debug_test_fail() {
    system_call(SystemCall::DebugTestFail);
    loop {}
}

fn system_call(message: SystemCall) -> SystemCall {
    let addr = task_buffer_addr();
    unsafe {
        let buffer = unsafe { &mut *(addr as *mut TaskBuffer) };
        buffer.call = Some(message);
        system_call_raw();
        buffer.call.take().unwrap()
    }
}

fn system_call_put_payload<T: Any>(message: SystemCall, payload: T) -> SystemCall {
    use core::mem::{size_of};
    let addr = task_buffer_addr();

    unsafe {
        let buffer = unsafe { &mut *(addr as *mut TaskBuffer) };
        buffer.call = Some(message);

        buffer.payload_length = size_of::<T>();
        let payload_addr = &mut buffer.payload_data as *mut _ as *mut T;
        let mut payload_data = &mut *payload_addr;
        *payload_data = payload;

        system_call_raw();
        buffer.call.take().unwrap()
    }
}

fn system_call_take_payload<T: Any + Clone>(message: SystemCall) -> (SystemCall, T) {
    use core::mem::{size_of};
    let addr = task_buffer_addr();

    unsafe {
        let buffer = unsafe { &mut *(addr as *mut TaskBuffer) };
        buffer.call = Some(message);

        system_call_raw();

        let payload_addr = &mut buffer.payload_data as *mut _ as *mut T;
        let payload_data = &*payload_addr;
        assert!(buffer.payload_length != 0 && buffer.payload_length == size_of::<T>());

        (buffer.call.take().unwrap(), payload_data.clone())
    }
}

#[inline(never)]
unsafe fn system_call_raw() {
    unsafe {
        asm!("int 80h"
             ::
             : "rax", "rbx", "rcx", "rdx",
               "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"
             : "volatile", "intel");
    }
}
