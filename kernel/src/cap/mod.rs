mod cpool;
mod untyped;
mod thread;

pub use self::cpool::{CPoolHalf, CPoolFull, CPool, MDB, MDBAddr};
pub use self::untyped::{UntypedHalf};
pub use self::thread::{TCBHalf, TCB};
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

pub enum Cap<'a> {
    CPool(CPoolFull<'a>),
}

impl<'a> Cap<'a> {
    pub fn mdbs(&self) -> &[MDB<'a>] {
        match self {
            &Cap::CPool(ref full) =>
                full.mdbs(),
        }
    }

    pub fn mdbs_mut(&mut self) -> &mut [MDB<'a>] {
        match self {
            &mut Cap::CPool(ref mut full) =>
                full.mdbs_mut(),
        }
    }
}

pub struct CapFull<Half, M> {
    half: Half,
    mdbs: M,
    deleted: bool,
}

impl<Half, M> CapFull<Half, M> {
    pub fn new(half: Half, mdbs: M) -> Self {
        CapFull {
            half: half,
            mdbs: mdbs,
            deleted: false,
        }
    }

    pub fn mark_deleted(&mut self) {
        self.deleted = true;
    }

    pub fn mdbs(&self) -> &M {
        &self.mdbs
    }

    pub fn mdbs_mut(&mut self) -> &mut M {
        &mut self.mdbs
    }
}

impl<Half, M> Drop for CapFull<Half, M> {
    fn drop(&mut self) {
        assert!(self.deleted, "attempt to drop unmarked CapFull.");
    }
}

pub trait CapHalf {
    fn mark_deleted(&mut self);
}

pub trait SystemCallable {
    fn handle_send(&mut self, CapSendMessage);
}

pub trait CapReadObject<T, U: Deref<Target=T>> {
    fn read(&self) -> U;
}

pub trait CapReadRefObject<'a, T, U: Deref<Target=T> + 'a> {
    fn read(&'a self) -> U;
}

pub trait CapWriteObject<T, U: Deref<Target=T> + DerefMut> {
    fn write(&mut self) -> U;
}

pub trait CapWriteRefObject<'a, T, U: Deref<Target=T> + DerefMut + 'a> {
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
