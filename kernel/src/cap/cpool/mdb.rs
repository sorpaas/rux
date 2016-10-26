use cap::{CPoolHalf, Cap, CPool, CapWriteObject, CapReadObject};
use util::{SharedReadGuard, SharedWriteGuard, RefGuard, RefMutGuard, IndexedSharedReadGuard, IndexedSharedWriteGuard, Streamer};
use core::ops::{Deref, DerefMut};
use core::marker::{PhantomData};

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
        MDB {
            this: None,
            first_child: None,
            parent: None,
            next: None,
            prev: None,
            holding_parent: Some(self),
        }
    }

    fn match_mdbs<'b>(cap: &'b mut Cap<'a>) -> &'b mut [MDB<'a>] {
        match cap {
            &mut Cap::CPool(ref mut cpool) => {
                &mut cpool.mdbs
            }
        }
    }

    pub unsafe fn set(&mut self, addr: MDBAddr) {
        self.this = Some(addr.clone());

        // Update prev
        if self.prev.is_some() {
            let mut prev = self.prev.clone().unwrap();
            let mut full_option = prev.cpool.write(prev.cpool_index);
            let full = full_option.as_mut().unwrap();
            let ref mut mdbs = Self::match_mdbs(full);
            mdbs[prev.mdb_index].next = Some(addr.clone());
        }

        // Update children
        let mut current_child_option = self.first_child.clone();
        while current_child_option.is_some() {
            let mut current_child = current_child_option.clone().unwrap();
            let mut full_option = current_child.cpool.write(current_child.cpool_index);
            let full = full_option.as_mut().unwrap();
            let ref mut mdbs = Self::match_mdbs(full);
            mdbs[current_child.mdb_index].parent = Some(addr.clone());
            current_child_option = mdbs[current_child.mdb_index].next.clone();
        }

        // Update next
        if self.next.is_some() {
            let mut next = self.next.clone().unwrap();
            let mut full_option = next.cpool.write(next.cpool_index);
            let full = full_option.as_mut().unwrap();
            let ref mut mdbs = Self::match_mdbs(full);
            mdbs[next.mdb_index].prev = Some(addr.clone());
        }

        // If holding_parent exists, then insert this to holding parent.
        if self.holding_parent.is_some() {
            let holding_parent = self.holding_parent.take().unwrap();
            if holding_parent.first_child.is_some() {
                let mut first_child = holding_parent.first_child.clone().unwrap();
                let mut full_option = first_child.cpool.write(first_child.cpool_index);
                let full = full_option.as_mut().unwrap();
                let ref mut mdbs = Self::match_mdbs(full);
                mdbs[first_child.mdb_index].prev = Some(addr.clone());
            }
            self.next = holding_parent.first_child.clone();
            self.parent = holding_parent.this.clone();
            holding_parent.first_child = Some(addr.clone());
        }
    }
}

// pub struct MDBChildIter<'a> {
//     next_child: Option<MDBAddr>,
//     current_cpool: &'a CPool<'a>,
//     current_cpool_half: CPoolHalf,
// }

// impl<'a> Streamer<'a> for MDBChildIter<'a> {
//     type Item = RefGuard<'a, Option<Cap<'a>>, IndexedSharedReadGuard<'a, Option<Cap<'a>>, usize, CPool<'a>>>;

//     fn next(&'a mut self) -> Option<Self::Item> {
//         if self.next_child.is_some() {
//             let next_child = self.next_child.take().unwrap();
//             if next_child.cpool == self.current_cpool_half {
//                 {
//                     let cpool = self.current_cpool;
//                     let ref mdb = cpool[next_child.cpool_index as usize].as_ref().unwrap().mdbs()[next_child.mdb_index];
//                     self.next_child = mdb.next.clone();
//                 }
//                 Some(RefGuard::Ref(&self.current_cpool[next_child.cpool_index as usize]))
//             } else {
//                 {
//                     let cpool = next_child.cpool.read();
//                     let ref mdb = cpool[next_child.cpool_index as usize].as_ref().unwrap().mdbs()[next_child.mdb_index];
//                     self.next_child = mdb.next.clone();
//                 }
//                 Some(RefGuard::Guard(IndexedSharedReadGuard::new(
//                     next_child.cpool.read(),
//                     next_child.cpool_index as usize
//                 )))
//             }
//         } else {
//             None
//         }
//     }
// }

// pub struct MDBChildIterMut<'a> {
//     next_child: Option<MDBAddr>,
//     current_cpool: &'a mut CPool<'a>,
//     current_cpool_half: CPoolHalf,
// }

// impl<'a> Streamer<'a> for MDBChildIterMut<'a> {
//     type Item = RefMutGuard<'a, Option<Cap<'a>>, IndexedSharedWriteGuard<'a, Option<Cap<'a>>, usize, CPool<'a>>>;

//     fn next(&'a mut self) -> Option<Self::Item> {
//         if self.next_child.is_some() {
//             let mut next_child = self.next_child.take().unwrap();
//             if next_child.cpool == self.current_cpool_half {
//                 {
//                     let ref cpool = self.current_cpool;
//                     let ref mdb = cpool[next_child.cpool_index as usize].as_ref().unwrap().mdbs()[next_child.mdb_index];
//                     self.next_child = mdb.next.clone();
//                 }
//                 let full = &mut self.current_cpool[next_child.cpool_index as usize];
//                 Some(RefMutGuard::Ref(full))
//             } else {
//                 {
//                     let cpool = next_child.cpool.read();
//                     let ref mdb = cpool[next_child.cpool_index as usize].as_ref().unwrap().mdbs()[next_child.mdb_index];
//                     self.next_child = mdb.next.clone();
//                 }
//                 Some(RefMutGuard::Guard(IndexedSharedWriteGuard::new(
//                     next_child.cpool.write(),
//                     next_child.cpool_index as usize
//                 )))
//             }
//         } else {
//             None
//         }
//     }
// }
