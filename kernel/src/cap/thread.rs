use common::*;
use cap::{Capability, CapReadObject, CapWriteObject, CapFull, MDB, CapNearlyFull,
          CPoolHalf, UntypedFull, SystemCallable, CapSendMessage};
use arch::{ThreadRuntime};
use core::mem::{size_of, align_of};
use core::fmt;
use util::{RwLock, SharedReadGuard, SharedWriteGuard, MemoryObject};

pub type TCBFull = CapFull<TCBHalf, [MDB; 1]>;
pub type TCBNearlyFull<'a> = CapNearlyFull<TCBHalf, [Option<&'a mut MDB>; 1]>;

type TCBMemoryObject = MemoryObject<RwLock<TCB>>;

#[derive(Debug)]
pub struct TCB {
    // cpool: Capability, FIXME
    runtime: ThreadRuntime
}

impl TCB {
    // pub fn cpool(&self) -> &Capability {
    //     &self.cpool
    // }

    // pub fn cpool_mut(&mut self) -> &mut Capability {
    //     &mut self.cpool
    // }

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

impl TCBFull {
    pub fn retype<'a>(untyped: &'a mut UntypedFull,
                  //cpool: CPoolHalf FIXME
                  ) -> TCBNearlyFull<'a> {
        let alignment = align_of::<TCB>();
        let length = size_of::<TCB>();
        let (start_paddr, mdb) = untyped.allocate(length, alignment);

        let mut cap = TCBHalf {
            start_paddr: start_paddr,
        };

        unsafe {
            let obj = TCBMemoryObject::new(cap.start_paddr);

            *obj.as_mut().unwrap() = RwLock::new(TCB {
                // cpool: Capability::CPool(cpool),
                runtime: ThreadRuntime::new(VAddr::from(0x0: u64),
                                            0b110,
                                            VAddr::from(0x0: u64))
            });
        }

        TCBNearlyFull::new(cap, [ mdb ])
    }
}

#[derive(Debug, Clone)]
pub struct TCBHalf {
    start_paddr: PAddr,
}

impl<'a> CapReadObject<TCB, SharedReadGuard<'a, TCB>> for TCBHalf {
    fn read<'b>(&'b self) -> SharedReadGuard<'a, TCB> {
        unsafe {
            SharedReadGuard::new(TCBMemoryObject::new(self.start_paddr))
        }
    }
}

impl<'a> CapWriteObject<TCB, SharedWriteGuard<'a, TCB>> for TCBHalf {
    fn write<'b>(&'b mut self) -> SharedWriteGuard<'a, TCB> {
        unsafe {
            SharedWriteGuard::new(TCBMemoryObject::new(self.start_paddr))
        }
    }
}

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

impl TCBHalf {
    pub unsafe fn switch_to(&mut self) {
        let cloned = self.clone();
        let runtime = {
            let tcb = self.read();
            tcb.runtime.clone()
        };
        runtime.switch_to(cloned);
    }
}
