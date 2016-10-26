use cap::{CPoolHalf, Cap, CPool, CapFull, CapWriteObject, CapReadObject};
use super::{CPoolMemoryObject};
use util::{SharedReadGuard, SharedWriteGuard, RefGuard, RefMutGuard, IndexedSharedReadGuard, IndexedSharedWriteGuard, Streamer};
use core::ops::{Deref, DerefMut};
use core::marker::{PhantomData};

#[derive(Clone)]
pub struct MDBAddr {
    cpool: CPoolHalf,
    cpool_index: u8,
    mdb_index: usize,
}

pub struct MDB<'a> {
    this: Option<MDBAddr>,
    first_child: Option<MDBAddr>,
    parent: Option<MDBAddr>,
    prev: Option<MDBAddr>,
    next: Option<MDBAddr>,
    holding_parent: Option<&'a mut MDB<'a>>
}

impl<'a> MDB<'a> {
    pub fn derive(&'a mut self) -> MDB<'a> {
        assert!(self.this.is_some());
        MDB {
            this: None,
            first_child: None,
            parent: None,
            next: None,
            prev: None,
            holding_parent: Some(self),
        }
    }

    // Constraint: addr.cpool refer to current_cpool
    pub unsafe fn set(&mut self, addr: MDBAddr, current_cpool: &'a mut CPool<'a>) {
        self.this = Some(addr.clone());

        // Update prev
        if self.prev.is_some() {
            let mut prev = self.prev.clone().unwrap();
            if prev.cpool == addr.cpool {
                let full = current_cpool[prev.cpool_index as usize].as_mut().unwrap();
                let ref mut mdb = full.mdbs_mut()[prev.mdb_index];
                mdb.next = Some(addr.clone());
            } else {
                let mut cpool = prev.cpool.write();
                let full = current_cpool[prev.cpool_index as usize].as_mut().unwrap();
                let ref mut mdb = full.mdbs_mut()[prev.mdb_index];
                mdb.next = Some(addr.clone());
            }
        }

        // Update children
        let mut current_child_option = self.first_child.clone();
        while current_child_option.is_some() {
            let mut current_child = current_child_option.clone().unwrap();

            if current_child.cpool == addr.cpool {
                let full = current_cpool[current_child.cpool_index as usize].as_mut().unwrap();
                let ref mut mdb = full.mdbs_mut()[current_child.mdb_index];
                mdb.parent = Some(addr.clone());
                current_child_option = mdb.next.clone();
            } else {
                let mut cpool = current_child.cpool.write();
                let full = cpool[current_child.cpool_index as usize].as_mut().unwrap();
                let ref mut mdb = full.mdbs_mut()[current_child.mdb_index];
                mdb.parent = Some(addr.clone());
                current_child_option = mdb.next.clone();
            }
        }

        // Update next
        if self.next.is_some() {
            let mut next = self.next.clone().unwrap();
            if next.cpool == addr.cpool {
                let full = current_cpool[next.cpool_index as usize].as_mut().unwrap();
                let ref mut mdb = full.mdbs_mut()[next.mdb_index];
                mdb.prev = Some(addr.clone());
            } else {
                let mut cpool = next.cpool.write();
                let full = cpool[next.cpool_index as usize].as_mut().unwrap();
                let ref mut mdb = full.mdbs_mut()[next.mdb_index];
                mdb.prev = Some(addr.clone());
            }
        }

        // If holding_parent exists, then insert this to holding parent.
        if self.holding_parent.is_some() {
            let holding_parent = self.holding_parent.take().unwrap();
            if holding_parent.first_child.is_some() {
                let mut first_child = holding_parent.first_child.clone().unwrap();
                if first_child.cpool == addr.cpool {
                    let full = current_cpool[first_child.cpool_index as usize].as_mut().unwrap();
                    let ref mut mdb = full.mdbs_mut()[first_child.mdb_index];
                    mdb.prev = Some(addr.clone());
                } else {
                    let mut cpool = first_child.cpool.write();
                    let full = cpool[first_child.cpool_index as usize].as_mut().unwrap();
                    let ref mut mdb = full.mdbs_mut()[first_child.mdb_index];
                    mdb.prev = Some(addr.clone());
                }
            }
            self.next = holding_parent.first_child.clone();
            self.parent = holding_parent.this.clone();
            holding_parent.first_child = Some(addr.clone());
        }
    }
}

pub struct MDBReadGuard<'a> {
    guard: SharedReadGuard<'a, CPool<'a>>,
    cpool_index: u8,
    mdb_index: usize,
}

pub struct MDBWriteGuard<'a> {
    guard: SharedWriteGuard<'a, CPool<'a>>,
    cpool_index: u8,
    mdb_index: usize,
}

impl<'a> Deref for MDBReadGuard<'a> {
    type Target = MDB<'a>;
    fn deref(&self) -> &MDB<'a> {
        &self.guard[self.cpool_index as usize].as_ref().unwrap().mdbs()[self.mdb_index]
    }
}

impl<'a> Deref for MDBWriteGuard<'a> {
    type Target = MDB<'a>;
    fn deref(&self) -> &MDB<'a> {
        &self.guard[self.cpool_index as usize].as_ref().unwrap().mdbs()[self.mdb_index]
    }
}

impl<'a> DerefMut for MDBWriteGuard<'a> {
    fn deref_mut(&mut self) -> &mut MDB<'a> {
        &mut self.guard[self.cpool_index as usize].as_mut().unwrap().mdbs_mut()[self.mdb_index]
    }
}

pub struct MDBChildIter<'a> {
    next_child: Option<MDBAddr>,
    current_cpool: &'a CPool<'a>,
    current_cpool_half: CPoolHalf,
}

impl<'a> Streamer<'a> for MDBChildIter<'a> {
    type Item = RefGuard<'a, Option<Cap<'a>>, IndexedSharedReadGuard<'a, Option<Cap<'a>>, usize, CPool<'a>>>;

    fn next(&'a mut self) -> Option<Self::Item> {
        if self.next_child.is_some() {
            let next_child = self.next_child.take().unwrap();
            if next_child.cpool == self.current_cpool_half {
                {
                    let cpool = self.current_cpool;
                    let ref mdb = cpool[next_child.cpool_index as usize].as_ref().unwrap().mdbs()[next_child.mdb_index];
                    self.next_child = mdb.next.clone();
                }
                Some(RefGuard::Ref(&self.current_cpool[next_child.cpool_index as usize]))
            } else {
                {
                    let cpool = next_child.cpool.read();
                    let ref mdb = cpool[next_child.cpool_index as usize].as_ref().unwrap().mdbs()[next_child.mdb_index];
                    self.next_child = mdb.next.clone();
                }
                Some(RefGuard::Guard(IndexedSharedReadGuard::new(
                    next_child.cpool.read(),
                    next_child.cpool_index as usize
                )))
            }
        } else {
            None
        }
    }
}

pub struct MDBChildIterMut<'a> {
    next_child: Option<MDBAddr>,
    current_cpool: &'a mut CPool<'a>,
    current_cpool_half: CPoolHalf,
}

impl<'a> Streamer<'a> for MDBChildIterMut<'a> {
    type Item = RefMutGuard<'a, Option<Cap<'a>>, IndexedSharedWriteGuard<'a, Option<Cap<'a>>, usize, CPool<'a>>>;

    fn next(&'a mut self) -> Option<Self::Item> {
        if self.next_child.is_some() {
            let mut next_child = self.next_child.take().unwrap();
            if next_child.cpool == self.current_cpool_half {
                {
                    let ref cpool = self.current_cpool;
                    let ref mdb = cpool[next_child.cpool_index as usize].as_ref().unwrap().mdbs()[next_child.mdb_index];
                    self.next_child = mdb.next.clone();
                }
                let full = &mut self.current_cpool[next_child.cpool_index as usize];
                Some(RefMutGuard::Ref(full))
            } else {
                {
                    let cpool = next_child.cpool.read();
                    let ref mdb = cpool[next_child.cpool_index as usize].as_ref().unwrap().mdbs()[next_child.mdb_index];
                    self.next_child = mdb.next.clone();
                }
                Some(RefMutGuard::Guard(IndexedSharedWriteGuard::new(
                    next_child.cpool.write(),
                    next_child.cpool_index as usize
                )))
            }
        } else {
            None
        }
    }
}
