mod table;
mod entry;
mod mapper;

use common::*;
use core::marker::PhantomData;

use super::PageBlockCapability;
use super::UntypedCapability;
use self::mapper::{Mapper, ActiveMapper};

pub trait PageTableStatus { }
pub enum ActivePageTableStatus { }
pub enum InactivePageTableStatus { }

impl PageTableStatus for ActivePageTableStatus { }
impl PageTableStatus for InactivePageTableStatus { }

pub struct PageTableCapability<L: PageTableStatus> {
    block_start_addr: PhysicalAddress,
    block_size: usize,
    table_start_addr: PhysicalAddress,
    active: PhantomData<L>,
}

pub type ActivePageTableCapability = PageTableCapability<ActivePageTableStatus>;
pub type InactivePageTableCapability = PageTableCapability<InactivePageTableStatus>;

impl<L> PageTableCapability<L> where L: PageTableStatus {
    pub fn map<T, U>(&self, block: &T, dest_addr: usize, untyped: UntypedCapability)
                     -> (VirtualAddress<U>, Option<UntypedCapability>)
        where T: PageBlockCapability<U> {
        use self::entry::PRESENT;

        let mut mapper = unsafe { ActiveMapper::new() };

        let virt = VirtualAddress::<U> {
            table_start_addr: self.table_start_addr,
            addr: dest_addr,
            _marker: PhantomData::<U>,
        };

        let untyped_r = unsafe {
            mapper.with(block.page_start_addr(), |mapper| {
                mapper.map_to(&virt, block, PRESENT, untyped)
            })
        };

        (virt, untyped_r)
    }

    pub fn unmap<U>(&self, addr: VirtualAddress<U>)
                    -> Option<UntypedCapability> {
        unimplemented!();
    }
}

impl ActivePageTableCapability {
    pub fn switch(new: InactivePageTableCapability, current: ActivePageTableCapability)
                  -> (ActivePageTableCapability, InactivePageTableCapability) {
        use x86::controlregs;

        let new_active = ActivePageTableCapability {
            block_start_addr: new.block_start_addr,
            block_size: new.block_size,
            table_start_addr: new.table_start_addr,
            active: PhantomData::<ActivePageTableStatus>,
        };

        let new_inactive = InactivePageTableCapability {
            block_start_addr: current.block_start_addr,
            block_size: current.block_size,
            table_start_addr: current.table_start_addr,
            active: PhantomData::<InactivePageTableStatus>,
        };

        unsafe {
            controlregs::cr3_write(new.block_start_addr as u64);
        }

        (new_active, new_inactive)
    }

    pub fn borrow<'r, U>(&'r self, virt: &VirtualAddress<U>) -> &'r U {
        assert!(virt.table_start_addr == self.table_start_addr);
        unsafe { &*(virt.addr as *mut _) }
    }

    pub fn borrow_mut<'r, U>(&'r self, virt: &mut VirtualAddress<U>) -> &'r U {
        assert!(virt.table_start_addr == self.table_start_addr);
        unsafe { &mut *(virt.addr as *mut _) }
    }
}

// WARNING: Currently it is unsafe to map one page block in one page table
// multiple times. It is indeed safe if that is not violated.
// TODO: Implement this.

pub struct VirtualAddress<T> {
    table_start_addr: PhysicalAddress,
    addr: usize,
    _marker: PhantomData<T>,
}

impl<T> Drop for VirtualAddress<T> {
    fn drop(&mut self) {
        unimplemented!()
    }
}
