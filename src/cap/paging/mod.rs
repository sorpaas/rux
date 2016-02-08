mod table;
mod mapper;
pub mod entry;

use common::*;
use core::marker::PhantomData;

use super::{MemoryBlockCapability, PageFrameCapability};
use super::UntypedCapability;
use super::utils;
use self::mapper::{Mapper, ActiveMapper};
use self::table::{PageTable, PageTableLevel4};

pub use self::entry::EntryFlags;

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
    pub fn map<T: PageFrameCapability, U>(&self, frame_cap: &T, dest_addr: usize, untyped: UntypedCapability)
                                          -> (VirtualAddress<U>, Option<UntypedCapability>) {
        let mut mapper = unsafe { ActiveMapper::new() };

        let virt = VirtualAddress::<U> {
            table_start_addr: self.table_start_addr,
            addr: dest_addr,
            _marker: PhantomData::<U>,
        };

        let untyped_q = unsafe {
            mapper.with(self.table_start_addr, |mapper| {
                let mut untyped_r = Some(untyped);
                for frame in frame_cap.frames() {
                    untyped_r = mapper.map_to(dest_addr + frame.offset * PAGE_SIZE, frame.addr, frame.flags,
                                              untyped_r.expect("Out of memory."))
                }
                untyped_r
            })
        };

        (virt, untyped_q)
    }

    pub fn identity_map<T: PageFrameCapability, U>(&self, frame_cap: &T, untyped: UntypedCapability)
                                                   -> (VirtualAddress<U>, Option<UntypedCapability>) {
        let first_frame = frame_cap.frames().next().expect("A frame capability must have at least one frame.");
        let dest_addr = first_frame.addr - first_frame.offset * PAGE_SIZE;
        self.map(frame_cap, dest_addr, untyped)
    }

    pub fn unmap<U>(&self, addr: VirtualAddress<U>)
                    -> Option<UntypedCapability> {
        unimplemented!();
    }
}

impl<L> Drop for PageTableCapability<L> where L: PageTableStatus {
    fn drop(&mut self) {
        unimplemented!();
    }
}

impl ActivePageTableCapability {
    pub fn borrow<'r, U>(&'r self, virt: &VirtualAddress<U>) -> &'r U {
        assert!(virt.table_start_addr == self.table_start_addr);
        unsafe { &*(virt.addr as *mut _) }
    }

    pub fn borrow_mut<'r, U>(&'r self, virt: &mut VirtualAddress<U>) -> &'r U {
        assert!(virt.table_start_addr == self.table_start_addr);
        unsafe { &mut *(virt.addr as *mut _) }
    }
}

impl InactivePageTableCapability {
    pub fn from_untyped(cap: UntypedCapability) -> (Option<InactivePageTableCapability>, Option<UntypedCapability>) {
        use self::entry::{PRESENT, WRITABLE};

        let block_start_addr = cap.block_start_addr();
        let frame_start_addr = utils::necessary_page_start_addr(block_start_addr);

        if frame_start_addr < block_start_addr {
            return (None, Some(cap));
        }

        let block_size = frame_start_addr - block_start_addr + PAGE_SIZE;

        if block_start_addr + block_size - 1 > cap.block_end_addr() {
            return (None, Some(cap));
        }

        let (mut u1, u2) = UntypedCapability::from_untyped(cap, block_size);
        assert!(u1.block_size() == block_size);

        u1.block_size = 0;

        // TODO Fix this.
        unsafe { (&mut *(frame_start_addr as *mut PageTable<PageTableLevel4>))[511].set_address(frame_start_addr, PRESENT | WRITABLE); }

        (Some(InactivePageTableCapability {
            block_start_addr: block_start_addr,
            block_size: block_size,
            table_start_addr: frame_start_addr,
            active: PhantomData::<InactivePageTableStatus>,
        }), u2)
    }
}

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

pub unsafe fn switch_to(new: InactivePageTableCapability) -> ActivePageTableCapability {
    use x86::controlregs;

    let new_active = ActivePageTableCapability {
        block_start_addr: new.block_start_addr,
        block_size: new.block_size,
        table_start_addr: new.table_start_addr,
        active: PhantomData::<ActivePageTableStatus>,
    };

    unsafe {
        controlregs::cr3_write(new.block_start_addr as u64);
    }

    new_active
}

pub struct Frame {
    addr: PhysicalAddress,
    offset: usize,
    flags: EntryFlags,
}

impl Frame {
    pub fn new(addr: PhysicalAddress, offset: usize, flags: EntryFlags) -> Frame {
        Frame {
            addr: addr,
            offset: offset,
            flags: flags,
        }
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
