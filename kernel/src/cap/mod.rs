mod cpool;
mod untyped;
mod thread;

pub use self::cpool::{CPoolHalf, CPoolFull, CPool, MDB, MDBAddr, CapFull, CapNearlyFull};
pub use self::untyped::{UntypedHalf, UntypedFull, UntypedNearlyFull};
pub use self::thread::{TCBHalf, TCBFull, TCB};
pub use abi::{CapSystemCall, CapSendMessage};
// pub use arch::{TopPageTableHalf, PageHalf, ArchSpecificCapability};

use common::*;
use core::ops::{Deref, DerefMut};

#[derive(Debug)]
pub enum Capability {
    Untyped(UntypedHalf),
    CPool(CPoolHalf),
    // TopPageTable(TopPageTableHalf),
    // Page(PageHalf),
    TCB(TCBHalf),
    // ArchSpecific(ArchSpecificCapability),
}

#[derive(Debug)]
pub enum Cap {
    CPool(CPoolFull),
    Untyped(UntypedFull),
    TCB(TCBFull),
}

impl Cap {
    pub unsafe fn set_mdb(&mut self, cpool: CPoolHalf, cpool_index: usize) {
        match self {
            &mut Cap::CPool(ref mut full) => full.set_mdb(cpool, cpool_index),
            &mut Cap::Untyped(ref mut full) => full.set_mdb(cpool, cpool_index),
            &mut Cap::TCB(ref mut full) => full.set_mdb(cpool, cpool_index),
        }
    }

    pub fn mdb(&self, index: usize) -> &MDB {
        match self {
            &Cap::CPool(ref full) => full.mdb(index),
            &Cap::Untyped(ref full) => full.mdb(index),
            &Cap::TCB(ref full) => full.mdb(index),
        }
    }

    pub fn mdb_mut(&mut self, index: usize) -> &mut MDB {
        match self {
            &mut Cap::CPool(ref mut full) => full.mdb_mut(index),
            &mut Cap::Untyped(ref mut full) => full.mdb_mut(index),
            &mut Cap::TCB(ref mut full) => full.mdb_mut(index),
        }
    }
}

impl From<CPoolFull> for Cap {
    fn from(full: CPoolFull) -> Cap {
        Cap::CPool(full)
    }
}

impl From<UntypedFull> for Cap {
    fn from(full: UntypedFull) -> Cap {
        Cap::Untyped(full)
    }
}

impl From<TCBFull> for Cap {
    fn from(full: TCBFull) -> Cap {
        Cap::TCB(full)
    }
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
