mod cpool;
mod untyped;
mod thread;

pub use self::cpool::{CPoolHalf};
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

pub trait CapReadObject<'a, T, U: Deref<Target=T> + 'a> {
    fn read(&'a self) -> U;
}

pub trait CapWriteObject<'a, T, U: Deref<Target=T> + DerefMut + 'a> {
    fn write(&'a mut self) -> U;
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
