mod mdb;
pub use self::mdb::{MDB, MDBAddr, CapFull};

use common::*;
use cap::{Cap, UntypedFull, CapReadObject};
use core::mem::{size_of, align_of};
use core::ops::{Index, IndexMut, Deref, DerefMut};
use core::slice::Iter;
use util::{RwLock, SharedReadGuard, SharedWriteGuard, MemoryObject};

pub type CPoolFull = CapFull<CPoolHalf, [MDB; 1]>;
pub type CPool = [RwLock<Option<Cap>>; 256];

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

        cap.insert(untyped.into());

        cap
    }

    pub fn retype(untyped: &mut UntypedFull) -> (CPoolHalf, [Option<&mut MDB>; 1]) {
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

        (cap, [ mdb ])
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

    pub fn insert(&mut self, mut cap: Cap) {
        let cpool = self.clone();
        for index in 0..256 {
            let mut item = self.try_write(index as u8);
            if item.is_some() {
                let mut item = item.unwrap();
                if item.is_none() {
                    unsafe { cap.set_mdb(cpool, index as u8); }
                    *item = Some(cap);
                    return;
                }
            }
        }
        assert!(false);
    }

    pub fn insert_half1<Half>(&mut self, half: Half, mut holdings: [Option<&mut MDB>; 1])
        where CapFull<Half, [MDB; 1]>: Into<Cap> {
        for i in 0..256 {
            let cpool = self.clone();
            let mut item = self.try_write(i as u8);
            if item.is_some() {
                let mut item = item.unwrap();
                if item.is_none() {
                    let mut cap = CapFull::new(half, [ MDB::default() ]);

                    let mut index = 0;
                    for hold in holdings.iter_mut() {
                        if hold.is_some() {
                            let hold = hold.take().unwrap();
                            unsafe { cap.set_mdb(cpool.clone(), i as u8); }
                            cap.mdb_mut(index).associate(hold);
                        }
                        index += 1;
                    }
                    *item = Some(cap.into());
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
