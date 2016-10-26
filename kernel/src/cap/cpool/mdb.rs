use cap::{CPoolHalf, Cap, CPool, CapWriteObject, CapReadObject};
use util::{SharedReadGuard, SharedWriteGuard, RefGuard, RefMutGuard, IndexedSharedReadGuard, IndexedSharedWriteGuard};
use core::ops::{Deref, DerefMut};
use core::marker::{PhantomData};

pub trait IntoFull<Half, M> where CapFull<Half, M>: Into<Cap> {
    unsafe fn into_full(mut self, cpool: CPoolHalf, cpool_index: usize) -> CapFull<Half, M>;
}

pub struct CapNearlyFull<Half, M> {
    half: Half,
    holdings: M
}

impl<Half, M> CapNearlyFull<Half, M> {
    pub fn new(half: Half, holdings: M) -> Self {
        CapNearlyFull {
            half: half,
            holdings: holdings
        }
    }
}

impl<'a, Half> IntoFull<Half, [MDB; 1]> for CapNearlyFull<Half, [Option<&'a mut MDB>; 1]>
    where CapFull<Half, [MDB; 1]>: Into<Cap> {
    unsafe fn into_full(mut self, cpool: CPoolHalf, cpool_index: usize) -> CapFull<Half, [MDB; 1]> {
        let mut cap = CapFull::new(self.half, [ MDB::default() ]);
        let mut index = 0;
        for hold in self.holdings.iter_mut() {
            unsafe { cap.set_mdb(cpool.clone(), cpool_index); }
            if hold.is_some() {
                let hold = hold.take().unwrap();
                cap.mdb_mut(index).associate(hold);
            }
            index += 1;
        }
        cap
    }
}

impl<Half, M> Deref for CapNearlyFull<Half, M> {
    type Target = Half;
    fn deref(&self) -> &Half {
        &self.half
    }
}

impl<Half, M> DerefMut for CapNearlyFull<Half, M> {
    fn deref_mut(&mut self) -> &mut Half {
        &mut self.half
    }
}

#[derive(Debug)]
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
}

impl<Half> CapFull<Half, [MDB; 1]> {
    pub unsafe fn set_mdb(&mut self, cpool: CPoolHalf, cpool_index: usize) {
        let mut mdb_index = 0;
        for mdb in self.mdbs.iter_mut() {
            mdb.set(MDBAddr {
                cpool: cpool.clone(),
                cpool_index: cpool_index,
                mdb_index: mdb_index,
            });
            mdb_index += 1;
        }
    }

    pub fn mdb(&self, index: usize) -> &MDB {
        &self.mdbs[index]
    }

    pub fn mdb_mut(&mut self, index: usize) -> &mut MDB {
        &mut self.mdbs[index]
    }
}

impl<'a, Half> IntoFull<Half, [MDB; 1]> for CapFull<Half, [MDB; 1]>
    where CapFull<Half, [MDB; 1]>: Into<Cap> {
    unsafe fn into_full(mut self, cpool: CPoolHalf, cpool_index: usize) -> CapFull<Half, [MDB; 1]> {
        self.set_mdb(cpool.clone(), cpool_index);
        self
    }
}

impl<Half, M> Deref for CapFull<Half, M> {
    type Target = Half;
    fn deref(&self) -> &Half {
        &self.half
    }
}

impl<Half, M> DerefMut for CapFull<Half, M> {
    fn deref_mut(&mut self) -> &mut Half {
        &mut self.half
    }
}

impl<Half, M> Drop for CapFull<Half, M> {
    fn drop(&mut self) {
        assert!(self.deleted, "attempt to drop unmarked CapFull.");
    }
}

#[derive(Clone, Debug)]
pub struct MDBAddr {
    cpool: CPoolHalf,
    cpool_index: usize,
    mdb_index: usize,
}

#[derive(Debug)]
pub struct MDB {
    this: Option<MDBAddr>,
    first_child: Option<MDBAddr>,
    parent: Option<MDBAddr>,
    prev: Option<MDBAddr>,
    next: Option<MDBAddr>
}

impl Default for MDB {
    fn default() -> Self {
        MDB {
            this: None,
            first_child: None,
            parent: None,
            prev: None,
            next: None
        }
    }
}

impl MDB {
    pub fn children<'a, 'b>(&'a self) -> MDBChildIter<'b> {
        MDBChildIter {
            next_child: self.first_child.clone(),
            _marker: PhantomData,
        }
    }

    pub fn children_mut<'a, 'b>(&'a mut self) -> MDBChildIterMut<'b> {
        MDBChildIterMut {
            next_child: self.first_child.clone(),
            _marker: PhantomData,
        }
    }

    pub fn associate(&mut self, holding_parent: &mut MDB) {
        assert!(self.parent.is_none() &&
                self.next.is_none() &&
                self.prev.is_none() &&
                self.this.is_some() &&
                holding_parent.this.is_some());

        if holding_parent.first_child.is_some() {
            let mut first_child = holding_parent.first_child.clone().unwrap();
            let mut full_option = first_child.cpool.write(first_child.cpool_index);
            let full = full_option.as_mut().unwrap();
            full.mdb_mut(first_child.mdb_index).prev = self.this.clone();
        }
        self.next = holding_parent.first_child.clone();
        self.parent = holding_parent.this.clone();
        holding_parent.first_child = self.this.clone();
    }

    pub unsafe fn set(&mut self, addr: MDBAddr) {
        self.this = Some(addr.clone());

        // Update prev
        if self.prev.is_some() {
            let mut prev = self.prev.clone().unwrap();
            let mut full_option = prev.cpool.write(prev.cpool_index);
            let full = full_option.as_mut().unwrap();
            full.mdb_mut(prev.mdb_index).next = Some(addr.clone());
        }

        // Update children
        let mut current_child_option = self.first_child.clone();
        while current_child_option.is_some() {
            let mut current_child = current_child_option.clone().unwrap();
            let mut full_option = current_child.cpool.write(current_child.cpool_index);
            let full = full_option.as_mut().unwrap();
            full.mdb_mut(current_child.mdb_index).parent = Some(addr.clone());
            current_child_option = full.mdb(current_child.mdb_index).next.clone();
        }

        // Update next
        if self.next.is_some() {
            let mut next = self.next.clone().unwrap();
            let mut full_option = next.cpool.write(next.cpool_index);
            let full = full_option.as_mut().unwrap();
            full.mdb_mut(next.mdb_index).prev = Some(addr.clone());
        }
    }
}

pub struct MDBChildIter<'a> {
    next_child: Option<MDBAddr>,
    _marker: PhantomData<SharedReadGuard<'a, Option<Cap>>>,
}

impl<'a> Iterator for MDBChildIter<'a> {
    type Item = SharedReadGuard<'a, Option<Cap>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_child.is_some() {
            let next_child = self.next_child.take().unwrap();
            let full_option = next_child.cpool.read(next_child.cpool_index);
            {
                let full = full_option.as_ref().unwrap();
                self.next_child = full.mdb(next_child.mdb_index).next.clone();
            }
            Some(full_option)
        } else {
            None
        }
    }
}

pub struct MDBChildIterMut<'a> {
    next_child: Option<MDBAddr>,
    _marker: PhantomData<SharedWriteGuard<'a, Option<Cap>>>,
}

impl<'a> Iterator for MDBChildIterMut<'a> {
    type Item = SharedWriteGuard<'a, Option<Cap>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_child.is_some() {
            let mut next_child = self.next_child.take().unwrap();
            let full_option = next_child.cpool.write(next_child.cpool_index);
            {
                let full = full_option.as_ref().unwrap();
                self.next_child = full.mdb(next_child.mdb_index).next.clone();
            }
            Some(full_option)
        } else {
            None
        }
    }
}
