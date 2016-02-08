pub mod table;
pub mod entry;

use common::*;
use core::marker::PhantomData;
use core::ptr::Unique;

use super::{MemoryBlockCapability, PageFrameCapability, BorrowableCapability};
use super::UntypedCapability;
use super::utils;
use super::utils::ContinuousFrameIterator;
// use self::mapper::{Mapper, ActiveMapper};
use self::table::{PageTable, PageTableLevel4};

use x86::controlregs;

pub use self::entry::EntryFlags;

pub trait PageTableStatus { }
pub enum ActivePageTableStatus { }
pub enum InactivePageTableStatus { }

impl PageTableStatus for ActivePageTableStatus { }
impl PageTableStatus for InactivePageTableStatus { }

pub struct PageTableCapability<L: PageTableStatus> {
    block_start_addr: PhysicalAddress,
    block_size: usize,
    frame_start_addr: PhysicalAddress,
    ignore_drop: bool,
    is_bootstrapping: bool,
    active: PhantomData<L>,
}

pub type ActivePageTableCapability = PageTableCapability<ActivePageTableStatus>;
pub type InactivePageTableCapability = PageTableCapability<InactivePageTableStatus>;

impl<L: PageTableStatus> MemoryBlockCapability for PageTableCapability<L> {
    fn block_start_addr(&self) -> PhysicalAddress {
        self.block_start_addr
    }

    fn block_size(&self) -> usize {
        self.block_size
    }
}

impl<L: PageTableStatus> PageFrameCapability for PageTableCapability<L> {
    type FrameIterator = ContinuousFrameIterator;
    fn frames(&self) -> ContinuousFrameIterator {
        use self::entry::WRITABLE;
        ContinuousFrameIterator::new(self.frame_start_addr(), 1, WRITABLE)
    }
}

impl<L: PageTableStatus> BorrowableCapability for PageTableCapability<L> {
    type Borrowable = PageTable<PageTableLevel4>;

    fn frame_start_addr(&self) -> PhysicalAddress {
        self.frame_start_addr
    }
}

impl ActivePageTableCapability {
    pub fn map<T: PageFrameCapability>(&self, frame_cap: &T, dest_addr: usize, untyped: UntypedCapability)
                                       -> (VirtualAddress, Option<UntypedCapability>) {
        use self::entry::PRESENT;

        let mut target_untyped = Some(untyped);
        let mut unique = unsafe { Unique::new(0xffffffff_fffff000 as *mut _) };
        let mut p4: &mut PageTable<PageTableLevel4> = unsafe { unique.get_mut() };
        for frame in frame_cap.frames() {
            let virt_addr = dest_addr + frame.offset * PAGE_SIZE;
            let page = Page::new(virt_addr);
            let untyped = target_untyped.expect("Out of memory.");

            let (mut p3, untyped) = p4.next_table_create(page.p4_index(), untyped);
            let untyped = untyped.expect("Out of memory.");
            let (mut p2, untyped) = p3.next_table_create(page.p3_index(), untyped);
            let untyped = untyped.expect("Out of memory.");
            let (mut p1, untyped) = p2.next_table_create(page.p2_index(), untyped);

            assert!(p1[page.p1_index()].is_unused());
            p1[page.p1_index()].set_address(frame.addr, frame.flags | PRESENT);

            target_untyped = untyped;
        }
        (VirtualAddress { table_addr: self.frame_start_addr(), addr: dest_addr }, target_untyped)
    }

    pub fn identity_map<T: PageFrameCapability>(&self, frame_cap: &T, untyped: UntypedCapability)
                                                   -> (VirtualAddress, Option<UntypedCapability>) {
        let first_frame = frame_cap.frames().next().expect("A frame capability must have at least one frame.");
        let dest_addr = first_frame.addr - first_frame.offset * PAGE_SIZE;
        self.map(frame_cap, dest_addr, untyped)
    }

    pub fn unmap<U>(&self, addr: VirtualAddress)
                    -> Option<UntypedCapability> {
        unimplemented!();
    }
}

impl InactivePageTableCapability {
    pub fn map<T: PageFrameCapability>(&self, frame_cap: &T, dest_addr: usize, untyped: UntypedCapability)
                                       -> (VirtualAddress, Option<UntypedCapability>) {

        use x86::{controlregs, tlb};
        let mut p4 = unsafe { &mut *(0xffffffff_fffff000 as *mut PageTable<PageTableLevel4>) };
        let flush_tlb = || unsafe { tlb::flush_all() };
        let mut target_untyped = Some(untyped);

        {
            use self::entry::{PRESENT, WRITABLE};

            let address511 = unsafe { controlregs::cr3() } as usize;
            let backup510 = p4[510].raw();

            p4[510].set_address(address511, PRESENT | WRITABLE);

            // overwrite recursive mapping
            p4[511].set_address(self.frame_start_addr(), PRESENT | WRITABLE);
            flush_tlb();

            // TODO Make sure the address doesn't refer to 511 or 510.
            for frame in frame_cap.frames() {
                let virt_addr = dest_addr + frame.offset * PAGE_SIZE;
                let page = Page::new(virt_addr);
                let untyped = target_untyped.expect("Out of memory.");

                let (mut p3, untyped) = p4.next_table_create(page.p4_index(), untyped);
                let untyped = untyped.expect("Out of memory.");
                let (mut p2, untyped) = p3.next_table_create(page.p3_index(), untyped);
                let untyped = untyped.expect("Out of memory.");
                let (mut p1, untyped) = p2.next_table_create(page.p2_index(), untyped);

                assert!(p1[page.p1_index()].is_unused());
                p1[page.p1_index()].set_address(frame.addr, frame.flags | PRESENT);

                target_untyped = untyped;
            }

            let mut original_p4 = unsafe { &mut *(0xffffff7f_bfdfe000 as *mut PageTable<PageTableLevel4>) };

            // restore recursive mapping to original p4 table
            original_p4[511].set_address(address511, PRESENT | WRITABLE);
            unsafe { original_p4[510].set_raw(backup510); }
            flush_tlb();
        }

        let virt = VirtualAddress { addr: dest_addr, table_addr: self.frame_start_addr() };

        (virt, target_untyped)
    }

    pub fn identity_map<T: PageFrameCapability>(&self, frame_cap: &T, untyped: UntypedCapability)
                                                   -> (VirtualAddress, Option<UntypedCapability>) {
        let first_frame = frame_cap.frames().next().expect("A frame capability must have at least one frame.");
        let dest_addr = first_frame.addr - first_frame.offset * PAGE_SIZE;
        self.map(frame_cap, dest_addr, untyped)
    }

    pub fn unmap<U>(&self, addr: VirtualAddress)
                    -> Option<UntypedCapability> {
        unimplemented!();
    }
}

impl<L> Drop for PageTableCapability<L> where L: PageTableStatus {
    fn drop(&mut self) {
        if self.ignore_drop {
            return;
        }
        unimplemented!();
    }
}

impl ActivePageTableCapability {
    pub fn switch(mut new: InactivePageTableCapability, mut current: ActivePageTableCapability)
                  -> (ActivePageTableCapability, InactivePageTableCapability) {
        use x86::controlregs;

        let new_active = ActivePageTableCapability {
            block_start_addr: new.block_start_addr,
            block_size: new.block_size,
            frame_start_addr: new.frame_start_addr,
            ignore_drop: new.ignore_drop,
            is_bootstrapping: new.is_bootstrapping,
            active: PhantomData::<ActivePageTableStatus>,
        };

        let new_inactive = InactivePageTableCapability {
            block_start_addr: current.block_start_addr,
            block_size: current.block_size,
            frame_start_addr: current.frame_start_addr,
            ignore_drop: new.ignore_drop,
            is_bootstrapping: new.is_bootstrapping,
            active: PhantomData::<InactivePageTableStatus>,
        };

        new.ignore_drop = true;
        current.ignore_drop = true;

        unsafe {
            controlregs::cr3_write(new.block_start_addr as u64);
        }

        (new_active, new_inactive)
    }

    pub unsafe fn bootstrap() -> ActivePageTableCapability {
        let frame_start_addr = unsafe { controlregs::cr3() } as usize;

        ActivePageTableCapability {
            block_start_addr: frame_start_addr,
            block_size: PAGE_SIZE,
            frame_start_addr: frame_start_addr,
            ignore_drop: true,
            is_bootstrapping: true,
            active: PhantomData::<ActivePageTableStatus>,
        }
    }
}

impl InactivePageTableCapability {
    pub fn from_untyped(cap: UntypedCapability, active: &ActivePageTableCapability) -> (Option<InactivePageTableCapability>, Option<UntypedCapability>) {
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

        let mut untyped = u2;
        let cap = InactivePageTableCapability {
            block_start_addr: block_start_addr,
            block_size: block_size,
            frame_start_addr: frame_start_addr,
            ignore_drop: false,
            is_bootstrapping: false,
            active: PhantomData::<InactivePageTableStatus>,
        };

        if active.is_bootstrapping {
            // The assembly code used 2Mib huge page for the initial mapping, so we do a little hack here to initialize the page table.
            let mut table = unsafe { &mut *(frame_start_addr as *mut PageTable<PageTableLevel4>) };
            table.zero();
            table[511].set_address(frame_start_addr, PRESENT | WRITABLE);
        } else {
            let (mut virt, ou) = active.map(&cap, 0x0, untyped.expect("Out of memory"));
            let mut table = cap.borrow_mut(&mut virt, active);
            table.zero();
            table[511].set_address(frame_start_addr, PRESENT | WRITABLE);
            untyped = ou;
            // TODO active.unmap(cap, virt);
        }


        (Some(cap), untyped)
    }
}

#[derive(Clone, Copy)]
struct Page {
    number: usize,
}

impl Page {
    pub fn new(address: usize) -> Page {
        assert!(address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000, "invalid address: 0x{:x}", address);
        Page { number: address / PAGE_SIZE }
    }

    fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
}

pub struct Frame {
    addr: PhysicalAddress,
    offset: usize,
    flags: EntryFlags,
}

pub struct VirtualAddress {
    table_addr: PhysicalAddress,
    addr: PhysicalAddress,
}

impl VirtualAddress {
    pub fn addr(&self) -> PhysicalAddress {
        self.addr
    }

    pub fn table_addr(&self) -> PhysicalAddress {
        self.table_addr
    }
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
