use common::*;
use super::{Capability, CapHalf};
use super::untyped::{UntypedHalf};
use arch::{ThreadRuntime};
use core::mem::{size_of, align_of};
use arch;

#[derive(Debug)]
pub struct TCB {
    cpool: Capability,
    runtime: ThreadRuntime
}

#[derive(Debug, Clone)]
pub struct TCBHalf {
    start_paddr: PAddr,
    deleted: bool
}

normal_half!(TCBHalf);

impl TCBHalf {
    // TODO handle data races
    pub fn with_tcb<Return, F: FnOnce(&TCB) -> Return>(&self, f: F) -> Return {
        unsafe {
            arch::with_object(self.start_paddr, |tcb: &TCB| {
                f(tcb)
            })
        }
    }

    // TODO handle data races
    pub fn with_tcb_mut<Return, F: FnOnce(&mut TCB) -> Return>(&mut self, f: F) -> Return {
        unsafe {
            arch::with_object_mut(self.start_paddr, |tcb: &mut TCB| {
                f(tcb)
            })
        }
    }

    pub fn new(cpool: Capability,
               runtime: ThreadRuntime,
               untyped: &mut UntypedHalf) -> TCBHalf {
        let alignment = align_of::<TCB>();
        let length = size_of::<TCB>();
        let start_paddr = untyped.allocate(length, alignment);

        let mut cap = TCBHalf {
            start_paddr: start_paddr,
            deleted: false
        };

        cap.with_tcb_mut(|tcb| {
            *tcb = TCB {
                cpool: cpool,
                runtime: runtime
            }
        });

        cap
    }
}
