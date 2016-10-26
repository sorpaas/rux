mod mdb;
pub use self::mdb::{MDB, MDBAddr, CapFull, IntoFull, CapNearlyFull};

use common::*;
use cap::{Cap, UntypedFull, UntypedNearlyFull, CapReadObject};
use core::mem::{size_of, align_of};
use core::ops::{Index, IndexMut, Deref, DerefMut};
use core::slice::Iter;
use util::{RwLock, SharedReadGuard, SharedWriteGuard, MemoryObject};

pub type CPoolFull = CapFull<CPoolHalf, [MDB; 1]>;
pub type CPoolNearlyFull<'a> = CapNearlyFull<CPoolHalf, [Option<&'a mut MDB>; 1]>;
pub type CPool = [RwLock<Option<Cap>>; 256];

macro_rules! cpool_default {
    () => ({
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
         RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None)]
    })
}

impl CPoolFull {
    pub unsafe fn bootstrap(mut untyped: UntypedFull) -> CPoolHalf {
        let alignment = align_of::<CPool>();
        let length = size_of::<CPool>();
        let (start_paddr, _) = untyped.allocate(length, alignment);

        let mut cap = CPoolHalf {
            start_paddr: start_paddr,
        };

        unsafe {
            let obj = MemoryObject::<CPool>::new(cap.start_paddr);
            *(obj.as_mut().unwrap()) =
                cpool_default!();
        }

        cap.insert(untyped);

        let mut untyped_cap = cap.write(0);
        let mut untyped = match untyped_cap.as_mut().unwrap() {
            &mut Cap::Untyped(ref mut untyped) => untyped,
            _ => panic!(),
        };
        let cloned_cap = cap.clone();
        cap.insert(CPoolNearlyFull::new(cloned_cap, [ Some(untyped.mdb_mut(0)) ]));

        cap
    }

    pub fn retype<'a>(untyped: &'a mut UntypedFull) -> CPoolNearlyFull<'a> {
        let alignment = align_of::<CPool>();
        let length = size_of::<CPool>();
        let (start_paddr, mdb) = untyped.allocate(length, alignment);

        let mut cap = CPoolHalf {
            start_paddr: start_paddr,
        };

        unsafe {
            let obj = MemoryObject::<CPool>::new(cap.start_paddr);
            *(obj.as_mut().unwrap()) =
                cpool_default!();
        }

        CPoolNearlyFull::new(cap, [ mdb ])
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

    pub fn insert<Half, M, U>(&mut self, mut cap: U)
        where U: IntoFull<Half, M>, Cap: From<CapFull<Half, M>> {
        let cpool = self.clone();
        for index in 0..256 {
            let mut item = self.try_write(index as u8);
            if item.is_some() {
                let mut item = item.unwrap();
                if item.is_none() {
                    *item = unsafe { Some(cap.into_full(cpool, index as u8).into()) };
                    return;
                }
            }
        }
        assert!(false);
    }

    fn item_paddr(&self, index: u8) -> PAddr {
        self.start_paddr + size_of::<RwLock<Option<Cap>>>() * (index as usize)
    }

    pub fn read<'a, 'b>(&'a self, index: u8) -> SharedReadGuard<'b, Option<Cap>> {
        let paddr = self.item_paddr(index);
        unsafe { SharedReadGuard::new(MemoryObject::<RwLock<Option<Cap>>>::new(paddr)) }
    }

    pub fn try_read<'a, 'b>(&'a self, index: u8) -> Option<SharedReadGuard<'b, Option<Cap>>> {
        let paddr = self.item_paddr(index);
        unsafe { SharedReadGuard::try_new(MemoryObject::<RwLock<Option<Cap>>>::new(paddr)) }
    }

    pub fn write<'a, 'b>(&'a mut self, index: u8) -> SharedWriteGuard<'b, Option<Cap>> {
        let paddr = self.item_paddr(index);
        unsafe { SharedWriteGuard::new(MemoryObject::<RwLock<Option<Cap>>>::new(paddr)) }
    }

    pub fn try_write<'a, 'b>(&'a mut self, index: u8) -> Option<SharedWriteGuard<'b, Option<Cap>>> {
        let paddr = self.item_paddr(index);
        unsafe { SharedWriteGuard::try_new(MemoryObject::<RwLock<Option<Cap>>>::new(paddr)) }
    }

    // pub fn traverse(&self, routes: &[u8]) -> SharedReadGuard<Option<Cap>> {
    //     if routes.len() == 0 {
    //         assert!(false);
    //     }
    //     let (first, rest) = routes.split_first().unwrap();
    //     let mut current_cap = self.read(*first);

    //     for path in rest {
    //         let new_current_cap = if let &Some(Cap::CPool(ref cpool_full)) = current_cap.deref() {
    //             Some(cpool_full.read(*path))
    //         } else {
    //             None
    //         };
    //         if new_current_cap.is_some() {
    //             current_cap = new_current_cap.unwrap();
    //         }
    //     }

    //     return current_cap;
    // }
}
