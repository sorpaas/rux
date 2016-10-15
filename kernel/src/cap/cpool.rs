use common::*;
use super::{Capability, CapHalf};
use super::untyped::{UntypedHalf};
use core::mem::{size_of, align_of};
use core::ops::{Index, IndexMut};
use core::slice::Iter;
use arch;

pub fn with_cspace<Return, F: FnOnce(&Option<Capability>) -> Return>(root_cap: &Capability,
                                                                     route: &[u8], f: F) -> Return {
    let target = {
        match root_cap {
            &Capability::CPool(ref target) => target,
            _ => return f(&None)
        }
    };
    let root = &target;

    let mut subcpool_half: Option<CPoolHalf> = None;
    let (route_last, route_cpool) = route.split_last().unwrap();

    let mut failed = false;

    for r in route_cpool {
        if failed {
            break;
        }

        subcpool_half = {
            let current = if subcpool_half.is_none() {
                root
            } else {
                subcpool_half.as_ref().unwrap()
            };

            current.with_cpool(|cpool| {
                let ref middle = cpool[*r as usize];

                match middle {
                    &Some(Capability::CPool(ref cpool)) => {
                        let mut subcpool = cpool.clone();
                        subcpool.mark_deleted();

                        Some(subcpool)
                    },
                    _ => {
                        failed = true;
                        None
                    }
                }
            })
        };
    }

    if failed {
        f(&None)
    } else {
        let current = if subcpool_half.is_none() {
            root
        } else {
            subcpool_half.as_ref().unwrap()
        };

        current.with_cpool(|cpool| {
            f(&cpool[*route_last as usize])
        })
    }
}

pub fn with_cspace_mut<Return, F: FnOnce(&mut Option<Capability>) -> Return>(root_cap: &mut Capability,
                                                                             route: &[u8], f: F) -> Return {
    let ref mut target = {
        match root_cap {
            &mut Capability::CPool(ref mut target) => target,
            _ => return f(&mut None)
        }
    };
    let mut target_mut = target;
    let mut root = &mut target_mut;

    let mut subcpool_half: Option<CPoolHalf> = None;
    let (route_last, route_cpool) = route.split_last().unwrap();

    let mut failed = false;

    for r in route_cpool {
        if failed {
            break;
        }

        subcpool_half = {
            let current = if subcpool_half.is_none() {
                root
            } else {
                subcpool_half.as_ref().unwrap()
            };

            current.with_cpool(|cpool| {
                let ref middle = cpool[*r as usize];

                match middle {
                    &Some(Capability::CPool(ref cpool)) => {
                        let mut subcpool = cpool.clone();
                        subcpool.mark_deleted();

                        Some(subcpool)
                    },
                    _ => {
                        failed = true;
                        None
                    }
                }
            })
        };
    }

    if failed {
        f(&mut None)
    } else {
        let current = if subcpool_half.is_none() {
            root
        } else {
            subcpool_half.as_mut().unwrap()
        };

        current.with_cpool_mut(|cpool| {
            f(&mut cpool[*route_last as usize])
        })
    }
}


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

impl CPoolHalf {
    pub fn with_cpool<Return, F: FnOnce(&CPool) -> Return>(&self, f: F) -> Return {
        unsafe {
            arch::with_object(self.start_paddr, |cpool: &CPool| {
                f(cpool)
            })
        }
    }

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
            start_paddr: start_paddr,
            deleted: false
        };

        cap.with_cpool_mut(|cpool| {
            *cpool = CPool([None, None, None, None, None, None, None, None,
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
                            None, None, None, None, None, None, None, None]);
        });

        cap
    }
}
