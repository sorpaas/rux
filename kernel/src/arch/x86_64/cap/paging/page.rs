use common::*;
use arch::paging::{BASE_PAGE_LENGTH};
use utils::{MemoryObject, UniqueMemoryGuard, Mutex, MutexGuard};
use cap::{UntypedHalf, Capability, CapObject, CapHalf};

/// Non-clonable, lock in CapHalf.

#[derive(Debug)]
pub struct PageHalf {
    start_paddr: PAddr,
    lock: Mutex<()>,
    deleted: bool
}

normal_half!(PageHalf);

impl<'a> CapObject<'a, [u8; BASE_PAGE_LENGTH], UniqueMemoryGuard<[u8; BASE_PAGE_LENGTH], MutexGuard<'a, ()>>> for PageHalf {
    fn lock(&mut self) -> UniqueMemoryGuard<[u8; BASE_PAGE_LENGTH], MutexGuard<()>> {
        unsafe { UniqueMemoryGuard::new(
            MemoryObject::<[u8; BASE_PAGE_LENGTH]>::new(self.start_paddr),
            self.lock.lock()
        ) }
    }
}

impl PageHalf {
    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }

    pub fn new(untyped: &mut UntypedHalf) -> PageHalf {
        let alignment = BASE_PAGE_LENGTH;
        let paddr = untyped.allocate(BASE_PAGE_LENGTH, alignment);

        let mut half = PageHalf {
            start_paddr: paddr,
            lock: Mutex::new(()),
            deleted: false
        };

        for u in half.lock().iter_mut() {
            *u = 0x0: u8;
        }

        half
    }

    pub fn length() -> usize {
        BASE_PAGE_LENGTH
    }
}
