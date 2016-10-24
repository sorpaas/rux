use common::*;
use super::{Capability, CapObject, CapHalf, CPoolHalf, UntypedHalf, SystemCallable, CapSendMessage};
use arch::{ThreadRuntime};
use core::mem::{size_of, align_of};
use core::fmt;
use utils::{Mutex, MutexMemoryGuard, MemoryObject};

#[derive(Debug)]
pub struct TCB {
    cpool: Capability,
    runtime: ThreadRuntime
}

impl TCB {
    pub fn cpool(&self) -> &Capability {
        &self.cpool
    }

    pub fn cpool_mut(&mut self) -> &mut Capability {
        &mut self.cpool
    }

    pub fn runtime(&self) -> &ThreadRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut ThreadRuntime {
        &mut self.runtime
    }

    pub fn set_instruction_pointer(&mut self, instruction_pointer:
                                   VAddr) {
        self.runtime_mut().set_instruction_pointer(instruction_pointer)
    }

    pub fn set_stack_pointer(&mut self, stack_pointer: VAddr) {
        self.runtime_mut().set_stack_pointer(stack_pointer)
    }
}

#[derive(Debug, Clone)]
pub struct TCBHalf {
    start_paddr: PAddr,
    deleted: bool
}

normal_half!(TCBHalf);

impl SystemCallable for TCBHalf {
    fn handle_send(&mut self, msg: CapSendMessage) {
        match msg {
            CapSendMessage::TCBYield => unsafe {
                log!("yielding to target tcb ...");
                self.switch_to()
            }
        }
    }
}

impl fmt::Display for TCBHalf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TCB(0x{:x})", self.start_paddr)
    }
}

impl<'a> CapObject<'a, TCB, MutexMemoryGuard<'a, TCB>> for TCBHalf {
    fn lock(&mut self) -> MutexMemoryGuard<TCB> {
        unsafe {
            let obj = MemoryObject::<Mutex<TCB>>::new(self.start_paddr);
            MutexMemoryGuard::new(obj)
        }
    }
}

impl TCBHalf {
    pub unsafe fn switch_to(&mut self) {
        let cloned = self.clone();
        let mut tcb = self.lock();
        tcb.runtime_mut().switch_to(cloned);
    }

    pub fn new(cpool: CPoolHalf,
               untyped: &mut UntypedHalf) -> TCBHalf {
        let alignment = align_of::<TCB>();
        let length = size_of::<TCB>();
        let start_paddr = untyped.allocate(length, alignment);

        let mut cap = TCBHalf {
            start_paddr: start_paddr,
            deleted: false
        };

        unsafe {
            let obj = MemoryObject::<Mutex<TCB>>::new(cap.start_paddr);

            // FIXME rust recognizes those initial zeros as a TCB with
            // a zero Untyped, which is incorrect. The zero Untyped is
            // considered dropped, so the drop function is called. It
            // is not marked yet, so this cause an error.
            // match (*obj.as_mut().unwrap()).cpool {
            //     Capability::Untyped(ref mut untyped) =>
            //         untyped.mark_deleted(),
            //     _ => assert!(false)
            // }

            *obj.as_mut().unwrap() = Mutex::new(TCB {
                cpool: Capability::CPool(cpool),
                runtime: ThreadRuntime::new(VAddr::from(0x0: u64),
                                            0b110,
                                            VAddr::from(0x0: u64))
            });
        }

        cap
    }
}
