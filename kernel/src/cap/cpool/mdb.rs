use cap::{CPoolHalf, Cap, CPool, CapWriteObject, CapReadObject};
use util::{SharedReadGuard, SharedWriteGuard, RefGuard, RefMutGuard, IndexedSharedReadGuard, IndexedSharedWriteGuard};
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

fn match_mdbs_mut<'a, 'b>(cap: &'b mut Cap<'a>) -> &'b mut [MDB<'a>] {
    match cap {
        &mut Cap::CPool(ref mut cpool) => {
            &mut cpool.mdbs
        }
    }
}

fn match_mdbs<'a, 'b>(cap: &'b Cap<'a>) -> &'b [MDB<'a>] {
    match cap {
        &Cap::CPool(ref cpool) => {
            &cpool.mdbs
        }
    }
}

impl<'a> MDB<'a> {
    pub fn children<'b>(&'a self) -> MDBChildIter<'b> {
        MDBChildIter {
            next_child: self.first_child.clone(),
            _marker: PhantomData,
        }
    }

    pub fn children_mut<'b>(&'a mut self) -> MDBChildIterMut<'b> {
        MDBChildIterMut {
            next_child: self.first_child.clone(),
            _marker: PhantomData,
        }
    }

    pub fn associate(&mut self, holding_parent: &'a mut MDB<'a>) {
        assert!(self.parent.is_none() &&
                self.next.is_none() &&
                self.prev.is_none() &&
                self.holding_parent.is_none() &&
                holding_parent.this.is_some());

        if self.this.is_none() {
            self.holding_parent = Some(holding_parent);
        } else {
            if holding_parent.first_child.is_some() {
                let mut first_child = holding_parent.first_child.clone().unwrap();
                let mut full_option = first_child.cpool.write(first_child.cpool_index);
                let full = full_option.as_mut().unwrap();
                let ref mut mdbs = match_mdbs_mut(full);
                mdbs[first_child.mdb_index].prev = self.this.clone();
            }
            self.next = holding_parent.first_child.clone();
            self.parent = holding_parent.this.clone();
            holding_parent.first_child = self.this.clone();
        }
    }

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

    pub unsafe fn set(&mut self, addr: MDBAddr) {
        self.this = Some(addr.clone());

        // Update prev
        if self.prev.is_some() {
            let mut prev = self.prev.clone().unwrap();
            let mut full_option = prev.cpool.write(prev.cpool_index);
            let full = full_option.as_mut().unwrap();
            let ref mut mdbs = match_mdbs_mut(full);
            mdbs[prev.mdb_index].next = Some(addr.clone());
        }

        // Update children
        let mut current_child_option = self.first_child.clone();
        while current_child_option.is_some() {
            let mut current_child = current_child_option.clone().unwrap();
            let mut full_option = current_child.cpool.write(current_child.cpool_index);
            let full = full_option.as_mut().unwrap();
            let ref mut mdbs = match_mdbs_mut(full);
            mdbs[current_child.mdb_index].parent = Some(addr.clone());
            current_child_option = mdbs[current_child.mdb_index].next.clone();
        }

        // Update next
        if self.next.is_some() {
            let mut next = self.next.clone().unwrap();
            let mut full_option = next.cpool.write(next.cpool_index);
            let full = full_option.as_mut().unwrap();
            let ref mut mdbs = match_mdbs_mut(full);
            mdbs[next.mdb_index].prev = Some(addr.clone());
        }

        // If holding_parent exists, then insert this to holding parent.
        if self.holding_parent.is_some() {
            let holding_parent = self.holding_parent.take().unwrap();
            if holding_parent.first_child.is_some() {
                let mut first_child = holding_parent.first_child.clone().unwrap();
                let mut full_option = first_child.cpool.write(first_child.cpool_index);
                let full = full_option.as_mut().unwrap();
                let ref mut mdbs = match_mdbs_mut(full);
                mdbs[first_child.mdb_index].prev = Some(addr.clone());
            }
            self.next = holding_parent.first_child.clone();
            self.parent = holding_parent.this.clone();
            holding_parent.first_child = Some(addr.clone());
        }
    }
}

pub struct MDBChildIter<'a> {
    next_child: Option<MDBAddr>,
    _marker: PhantomData<Option<Cap<'a>>>
}

impl<'a> Iterator for MDBChildIter<'a> {
    type Item = SharedReadGuard<'a, Option<Cap<'a>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_child.is_some() {
            let next_child = self.next_child.take().unwrap();
            let full_option = next_child.cpool.read(next_child.cpool_index);
            {
                let full = full_option.as_ref().unwrap();
                let ref mdbs = match_mdbs(full);
                self.next_child = mdbs[next_child.mdb_index].next.clone();
            }
            Some(full_option)
        } else {
            None
        }
    }
}

pub struct MDBChildIterMut<'a> {
    next_child: Option<MDBAddr>,
    _marker: PhantomData<Option<Cap<'a>>>
}

impl<'a> Iterator for MDBChildIterMut<'a> {
    type Item = SharedWriteGuard<'a, Option<Cap<'a>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_child.is_some() {
            let mut next_child = self.next_child.take().unwrap();
            let full_option = next_child.cpool.write(next_child.cpool_index);
            {
                let full = full_option.as_ref().unwrap();
                let ref mdbs = match_mdbs(full);
                self.next_child = mdbs[next_child.mdb_index].next.clone();
            }
            Some(full_option)
        } else {
            None
        }
    }
}
