use common::*;
use super::{Capability, CapHalf, CapReadObject, CapWriteObject};
use super::untyped::{UntypedHalf};
use core::mem::{size_of, align_of};
use core::ops::{Index, IndexMut};
use core::slice::Iter;
use util::{RwLock, SharedReadGuard, SharedWriteGuard, MemoryObject};

type CPoolMemoryObject = MemoryObject<RwLock<CPool>>;

#[derive(Debug, Clone)]
pub struct CPoolHalf {
    start_paddr: PAddr,
    deleted: bool
}

pub struct CPool([Option<Capability>; 256]);

impl Index<usize> for CPool {
    type Output = Option<Capability>;

    fn index<'a>(&'a self, _index: usize) -> &'a Option<Capability> {
        self.0.index(_index)
    }
}

impl IndexMut<usize> for CPool {
    fn index_mut<'a>(&'a mut self, _index: usize) -> &'a mut Option<Capability> {
        self.0.index_mut(_index)
    }
}

impl CPool {
    pub fn insert(&mut self, cap: Capability) {
        for space in self.0.iter_mut() {
            if space.is_none() {
                *space = Some(cap);
                return;
            }
        }
        assert!(false);
    }

    pub fn slice(&self) -> &[Option<Capability>] {
        &self.0
    }

    pub fn slice_mut(&mut self) -> &mut [Option<Capability>] {
        &mut self.0
    }
}

normal_half!(CPoolHalf);

impl<'a> CapReadObject<'a, CPool, SharedReadGuard<'a, CPool>> for CPoolHalf {
    fn read(&self) -> SharedReadGuard<CPool> {
        unsafe {
            SharedReadGuard::new(CPoolMemoryObject::new(self.start_paddr))
        }
    }
}

impl<'a> CapWriteObject<'a, CPool, SharedWriteGuard<'a, CPool>> for CPoolHalf {
    fn write(&mut self) -> SharedWriteGuard<CPool> {
        unsafe {
            SharedWriteGuard::new(CPoolMemoryObject::new(self.start_paddr))
        }
    }
}

impl CPoolHalf {
    pub fn traverse(&self, routes: &[u8]) -> Option<CPoolHalf> {
        let mut current_half = self.clone();
        current_half.mark_deleted();

        for path in routes {
            let mut cpool_half = current_half.clone();
            let cpool: SharedReadGuard<CPool> = cpool_half.read();
            match cpool[*path as usize] {
                Some(Capability::CPool(ref cpool_half)) => {
                    current_half = cpool_half.clone();
                    current_half.mark_deleted();
                },
                _ => {
                    return None;
                }
            }
        }

        Some(current_half)
    }

    pub fn new(untyped: &mut UntypedHalf) -> CPoolHalf {
        let alignment = align_of::<CPool>();
        let length = size_of::<CPool>();
        let start_paddr = untyped.allocate(length, alignment);

        let mut cap = CPoolHalf {
            start_paddr: start_paddr,
            deleted: false
        };

        unsafe {
            let obj = CPoolMemoryObject::new(cap.start_paddr);
            *(obj.as_mut().unwrap()) =
                  RwLock::new(
                      CPool([None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None,
                             None, None, None, None, None, None, None, None]));
        }

        cap
    }
}
