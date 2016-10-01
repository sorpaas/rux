use common::*;
use super::{Capability};
use super::untyped::{UntypedHalf};
use core::mem::{size_of, align_of};
use core::ops::{Index, IndexMut};
use core::slice::Iter;
use arch;

#[derive(Debug, Clone)]
pub struct CPoolHalf {
    start_paddr: PAddr
}

#[derive(Debug)]
pub struct CPool([Option<Capability>; 16]);

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

impl CPoolHalf {
    // TODO handle data races
    pub fn with_cpool<Return, F: FnOnce(&CPool) -> Return>(&self, f: F) -> Return {
        unsafe {
            arch::with_object(self.start_paddr, |cpool: &CPool| {
                f(cpool)
            })
        }
    }

    // TODO handle data races
    pub fn with_cpool_mut<Return, F: FnOnce(&mut CPool) -> Return>(&mut self, f: F) -> Return {
        unsafe {
            arch::with_object_mut(self.start_paddr, |cpool: &mut CPool| {
                f(cpool)
            })
        }
    }

    pub fn new(untyped: &mut UntypedHalf) -> CPoolHalf {
        let alignment = align_of::<CPool>();
        let length = size_of::<CPool>();
        let start_paddr = untyped.allocate(length, alignment);

        let mut cap = CPoolHalf {
            start_paddr: start_paddr
        };

        cap.with_cpool_mut(|cpool| {
            *cpool = CPool([None, None, None, None, None, None, None, None,
                            None, None, None, None, None, None, None, None]);
        });

        cap
    }
}
