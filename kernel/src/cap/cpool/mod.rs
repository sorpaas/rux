mod mdb;
pub use self::mdb::{MDB, MDBAddr};

use common::*;
use super::{Cap, CapHalf, CapReadObject, CapWriteObject, CapFull};
use super::untyped::{UntypedHalf};
use core::mem::{size_of, align_of};
use core::ops::{Index, IndexMut};
use core::slice::Iter;
use util::{RwLock, SharedReadGuard, SharedWriteGuard, MemoryObject};

pub type CPoolFull<'a> = CapFull<CPoolHalf, [MDB<'a>; 1]>;
pub type CPoolMemoryObject<'a> = MemoryObject<RwLock<CPool<'a>>>;

#[derive(Debug, Clone, PartialEq)]
pub struct CPoolHalf {
    start_paddr: PAddr
}

pub struct CPool<'a>([Option<Cap<'a>>; 256]);

impl<'a> Index<usize> for CPool<'a> {
    type Output = Option<Cap<'a>>;

    fn index(&self, _index: usize) -> &Option<Cap<'a>> {
        self.0.index(_index)
    }
}

impl<'a> IndexMut<usize> for CPool<'a> {
    fn index_mut(&mut self, _index: usize) -> &mut Option<Cap<'a>> {
        self.0.index_mut(_index)
    }
}

impl<'a> CPool<'a> {
    pub fn insert(&mut self, cap: Cap<'a>) {
        for space in self.0.iter_mut() {
            if space.is_none() {
                *space = Some(cap);
                return;
            }
        }
        assert!(false);
    }

    pub fn slice(&self) -> &[Option<Cap<'a>>] {
        &self.0
    }

    pub fn slice_mut(&mut self) -> &mut [Option<Cap<'a>>] {
        &mut self.0
    }
}

impl<'a> CapReadObject<CPool<'a>, SharedReadGuard<'a, CPool<'a>>> for CPoolHalf {
    fn read<'b>(&'b self) -> SharedReadGuard<'a, CPool<'a>> {
        unsafe {
            SharedReadGuard::new(CPoolMemoryObject::new(self.start_paddr))
        }
    }
}

impl<'a> CapWriteObject<CPool<'a>, SharedWriteGuard<'a, CPool<'a>>> for CPoolHalf {
    fn write<'b>(&'b mut self) -> SharedWriteGuard<'a, CPool<'a>> {
        unsafe {
            SharedWriteGuard::new(CPoolMemoryObject::new(self.start_paddr))
        }
    }
}

impl CPoolHalf {
    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }

    pub fn traverse(&self, routes: &[u8]) -> Option<CPoolHalf> {
        let mut current_half = self.clone();

        for path in routes {
            let mut cpool_half = current_half.clone();
            let cpool: SharedReadGuard<CPool> = cpool_half.read();
            match cpool[*path as usize] {
                Some(Cap::CPool(ref cpool_half)) => {
                    current_half = cpool_half.half.clone();
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
