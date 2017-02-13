use common::*;
use core::convert::{AsRef};
use core::ops::{Deref, DerefMut};
use cap::{self, UntypedCap, CPoolCap, CPoolDescriptor, RawPageCap, TaskBufferPageCap, TopPageTableCap, TaskCap, TaskDescriptor, TaskStatus, ChannelCap, ChannelDescriptor, ChannelValue, PAGE_LENGTH};
use abi::{SystemCall, TaskBuffer};

/// System call handling function. Dispatch based on the type of the
/// system call.
pub fn handle(call: &mut SystemCall, task_cap: TaskCap, cpool: CPoolCap) {
    match call {
        &mut SystemCall::Print {
            request: ref request
        } => {
            use core::str;
            let buffer = request.0.clone();
            let slice = &buffer[0..request.1];
            let s = str::from_utf8(slice).unwrap();
            log!("Userspace print: {}", s);
        },
        &mut SystemCall::CPoolListDebug => {
            for i in 0..256 {
                let arc = cpool.lookup_upgrade_any(CAddr::from(i));
                if arc.is_some() {
                    let arc = arc.unwrap();
                    if arc.is::<CPoolCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): CPoolCap);
                    } else if arc.is::<UntypedCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): UntypedCap);
                    } else if arc.is::<TaskCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): TaskCap);
                    } else if arc.is::<RawPageCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): RawPageCap);
                    } else if arc.is::<TaskBufferPageCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): TaskBufferPageCap);
                    } else if arc.is::<TopPageTableCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): TopPageTableCap);
                    } else if arc.is::<ChannelCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): ChannelCap);
                    } else {
                        log!("CPool index {} (arch specific) => {:?}", i, arc);
                        cap::drop_any(arc);
                    }
                }
            }
        },
        &mut SystemCall::RetypeCPool {
            request: ref request,
        } => {
            let source: Option<UntypedCap> = cpool.lookup_upgrade(request.0);
            if source.is_some() {
                let source = source.unwrap();
                let target = CPoolCap::retype_from(source.write().deref_mut());
                let result = cpool.lookup_downgrade_at(&target, request.1);
            }
        },
        &mut SystemCall::RetypeTask {
            request: ref request,
        } => {
            let source: Option<UntypedCap> = cpool.lookup_upgrade(request.0);
            if source.is_some() {
                let source = source.unwrap();
                let target = TaskCap::retype_from(source.write().deref_mut());
                let result = cpool.lookup_downgrade_at(&target, request.1);
            }
        },
        &mut SystemCall::TaskSetInstructionPointer {
            request: ref request,
        } => {
            let target: Option<TaskCap> = cpool.lookup_upgrade(request.0);
            if target.is_some() {
                let target = target.unwrap();
                target.write().set_instruction_pointer(VAddr::from(request.1));
            }
        },
        &mut SystemCall::TaskSetStackPointer {
            request: ref request,
        } => {
            let target: Option<TaskCap> = cpool.lookup_upgrade(request.0);
            if target.is_some() {
                let target = target.unwrap();
                target.write().set_stack_pointer(VAddr::from(request.1));
            }
        },
        &mut SystemCall::TaskSetCPool {
            request: ref request,
        } => {
            let target_task: TaskCap = cpool.lookup_upgrade(request.0).unwrap();
            let target_cpool: CPoolCap = cpool.lookup_upgrade(request.1).unwrap();
            target_task.read().downgrade_cpool(&target_cpool);
        },
        &mut SystemCall::TaskSetTopPageTable {
            request: ref request,
        } => {
            let target_task: TaskCap = cpool.lookup_upgrade(request.0).unwrap();
            let target_table: TopPageTableCap = cpool.lookup_upgrade(request.1).unwrap();
            target_task.read().downgrade_top_page_table(&target_table);
        },
        &mut SystemCall::TaskSetBuffer {
            request: ref request,
        } => {
            let target_task: TaskCap = cpool.lookup_upgrade(request.0).unwrap();
            let target_buffer: TaskBufferPageCap = cpool.lookup_upgrade(request.1).unwrap();
            target_task.read().downgrade_buffer(&target_buffer);
        },
        &mut SystemCall::TaskSetActive {
            request: ref request,
        } => {
            let target_task: TaskCap = cpool.lookup_upgrade(*request).unwrap();
            target_task.write().set_status(TaskStatus::Active);
        },
        &mut SystemCall::TaskSetInactive {
            request: ref request,
        } => {
            let target_task: TaskCap = cpool.lookup_upgrade(*request).unwrap();
            target_task.write().set_status(TaskStatus::Inactive);
        },
        &mut SystemCall::ChannelTake {
            request: ref request,
            response: ref mut response,
        } => {
            let mut chan_option: Option<ChannelCap> = cpool.lookup_upgrade(*request);
            if let Some(chan) = chan_option {
                task_cap.write().set_status(TaskStatus::ChannelWait(chan))
            }
        },
        &mut SystemCall::ChannelPut {
            request: ref request,
        } => {
            let chan_option: Option<ChannelCap> = cpool.lookup_upgrade(request.0);
            if let Some(chan) = chan_option {
                let value = ChannelValue::from_message(request.1.clone(), cpool.clone());
                if value.is_some() {
                    chan.write().put(value.unwrap());
                }
            }
        }
    }
}
