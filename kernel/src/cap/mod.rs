mod cpool;
mod untyped;
mod thread;

pub use self::cpool::{CPoolHalf, with_cspace, with_cspace_mut};
pub use self::untyped::{UntypedHalf};
pub use self::thread::{TCBHalf};
pub use abi::{CapSystemCall, CapSendMessage};
pub use arch::{TopPageTableHalf, PageHalf, ArchSpecificCapability};

use common::*;
use core::ops::{Deref, DerefMut};

#[derive(Debug)]
pub enum Capability {
    Untyped(UntypedHalf),
    CPool(CPoolHalf),
    TopPageTable(TopPageTableHalf),
    Page(PageHalf),
    TCB(TCBHalf),
    ArchSpecific(ArchSpecificCapability),
}

pub trait CapHalf {
    fn mark_deleted(&mut self);
}

pub trait SystemCallable {
    fn handle_send(&mut self, CapSendMessage);
}

pub trait CapObject<'a, T, U: Deref<Target=T> + DerefMut + 'a> {
    fn lock(&'a mut self) -> U;
}

pub trait CapReadonlyObject<'a, T, U: Deref<Target=T> + 'a> {
    fn lock(&'a self) -> U;
}

impl Capability {
}

impl SystemCallable for Capability {
    fn handle_send(&mut self, msg: CapSendMessage) {
        match self {
            &mut Capability::TCB(ref mut tcb) => {
                tcb.handle_send(msg);
            },
            _ => {
                log!("system call error: unhandled message");
            }
        }
    }
}
