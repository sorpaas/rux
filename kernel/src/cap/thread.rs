use common::*;
use super::{Capability, CapHalf, CPoolHalf, UntypedHalf};
use arch::{ThreadRuntime};
use core::mem::{size_of, align_of};
use arch;

#[derive(Debug)]
pub struct TCB {
    cpool: Capability,
    runtime: ThreadRuntime
}

impl TCB {
    pub fn runtime_mut(&mut self) -> &mut ThreadRuntime {
        &mut self.runtime
    }
}

#[derive(Debug, Clone)]
pub struct TCBHalf {
    start_paddr: PAddr,
    deleted: bool
}

normal_half!(TCBHalf);

impl TCBHalf {
    pub fn with_tcb<Return, F: FnOnce(&TCB) -> Return>(&self, f: F) -> Return {
        unsafe {
            arch::with_object(self.start_paddr, |tcb: &TCB| {
                f(tcb)
            })
        }
    }

    pub fn with_tcb_mut<Return, F: FnOnce(&mut TCB) -> Return>(&mut self, f: F) -> Return {
        unsafe {
            arch::with_object_mut(self.start_paddr, |tcb: &mut TCB| {
                f(tcb)
            })
        }
    }

    pub unsafe fn switch_to(&mut self) {
        let cloned = self.clone();
        self.with_tcb_mut(|tcb| {
            tcb.runtime_mut().switch_to(cloned);
        });
    }

    pub fn new(cpool: CPoolHalf,
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
            // FIXME rust recognizes those initial zeros as a TCB with
            // a zero Untyped, which is incorrect. The zero Untyped is
            // considered dropped, so the drop function is called. It
            // is not marked yet, so this cause an error.

            match (*tcb).cpool {
                Capability::Untyped(ref mut untyped) =>
                    untyped.mark_deleted(),
                _ => assert!(false)
            }

            *tcb = TCB {
                cpool: Capability::CPool(cpool),
                runtime: runtime
            }
        });

        cap
    }
}
