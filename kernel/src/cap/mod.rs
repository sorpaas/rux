mod cpool;
mod untyped;
mod thread;

pub use self::cpool::{CPoolHalf};
pub use self::untyped::{UntypedHalf};
pub use self::thread::{TCBHalf};

use common::*;

pub use arch::{TopPageTableHalf, PageHalf, ArchSpecificCapability};

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

impl Capability {
}
