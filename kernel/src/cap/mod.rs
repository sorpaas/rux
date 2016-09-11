mod cpool;
mod untyped;

pub use self::cpool::{CPoolHalf};
pub use self::untyped::{UntypedHalf};

use common::*;

pub use arch::{TopPageTableHalf, PageHalf, ArchSpecificCapability};

#[derive(Debug)]
pub enum Capability {
    Untyped(UntypedHalf),
    CPool(CPoolHalf),
    TopPageTable(TopPageTableHalf),
    Page(PageHalf),
    ArchSpecific(ArchSpecificCapability),
}

impl Capability {
}
