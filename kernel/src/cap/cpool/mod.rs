mod mdb;
pub use self::mdb::{MDB, MDBAddr, CapFull};

use common::*;
use cap::{Cap, UntypedFull, CapReadObject};
use core::mem::{size_of, align_of};
use core::ops::{Index, IndexMut, Deref, DerefMut};
use core::slice::Iter;
use util::{RwLock, SharedReadGuard, SharedWriteGuard, MemoryObject};

pub type CPoolFull<'a> = CapFull<CPoolHalf, [MDB<'a>; 1]>;
pub type CPool<'a> = [RwLock<Option<Cap<'a>>>; 256];

impl<'a> CPoolFull<'a> {
    pub fn retype(untyped: &'a mut UntypedFull<'a>) -> CPoolFull<'a> {
        let alignment = align_of::<CPool>();
        let length = size_of::<CPool>();
        let (start_paddr, mdb) = untyped.allocate(length, alignment);

        let mut cap = CPoolHalf {
            start_paddr: start_paddr,
        };

        unsafe {
            let obj = MemoryObject::<CPool>::new(cap.start_paddr);
            *(obj.as_mut().unwrap()) =
                [RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None),
                 RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None)];
        }

        Self::new(cap, [ mdb ])
    }
}

#[derive(Debug, Clone)]
pub struct CPoolHalf {
    start_paddr: PAddr
}

impl CPoolHalf {
    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }

    pub fn insert<'a, 'b>(&'b mut self, mut cap: Cap<'a>) {
        for i in 0..256 {
            let mut item = self.write(i);
            if item.is_none() {
                unsafe { cap.set_mdb(self.clone(), i); }
                *item = Some(cap);
                return;
            }
        }
        assert!(false);
    }

    fn item_paddr<'a, 'b>(&'b self, index: u8) -> PAddr {
        self.start_paddr + size_of::<RwLock<Option<Cap<'a>>>>() * (index as usize)
    }

    pub fn read<'a, 'b>(&'b self, index: u8) -> SharedReadGuard<'a, Option<Cap<'a>>> {
        let paddr = self.item_paddr(index);
        unsafe { SharedReadGuard::new(MemoryObject::<RwLock<Option<Cap<'a>>>>::new(paddr)) }
    }

    pub fn write<'a, 'b>(&'b mut self, index: u8) -> SharedWriteGuard<'a, Option<Cap<'a>>> {
        let paddr = self.item_paddr(index);
        unsafe { SharedWriteGuard::new(MemoryObject::<RwLock<Option<Cap<'a>>>>::new(paddr)) }
    }

    pub fn traverse<'a, 'b>(&'b self, routes: &[u8]) -> SharedReadGuard<'a, Option<Cap<'a>>> {
        if routes.len() == 0 {
            assert!(false);
        }
        let (first, rest) = routes.split_first().unwrap();
        let mut current_cap = self.read(*first);

        for path in rest {
            let new_current_cap = if let &Some(Cap::CPool(ref cpool_full)) = current_cap.deref() {
                Some(cpool_full.read(*path))
            } else {
                None
            };
            if new_current_cap.is_some() {
                current_cap = new_current_cap.unwrap();
            }
        }

        return current_cap;
    }
}
