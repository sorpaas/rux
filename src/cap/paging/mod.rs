pub mod table;
pub mod entry;

use common::*;
use core::marker::PhantomData;
use core::ptr::Unique;

use super::utils;
use super::{UntypedCapability, MemoryBlock,
            FrameCapability};
use self::table::{PageTable, PageTableLevel4};

use x86::controlregs;

pub struct PageTableCapability {
    p4_block: MemoryBlock
}

impl PageTableCapability {
    pub fn p4_block(&self) -> &MemoryBlock {
        &self.p4_block
    }

    pub fn map(&self, frame: FrameCapability, dest_addr: usize, untyped: UntypedCapability)
               -> (VirtualAddress, Option<UntypedCapability>) {
        if unsafe { controlregs::cr3() as usize } == self.p4_block().start_addr() {
            self.map_active(frame, dest_addr, untyped)
        } else {
            self.map_inactive(frame, dest_addr, untyped)
        }
    }

    fn map_active(&self, frame: FrameCapability, dest_addr: usize, untyped: UntypedCapability)
                  -> (VirtualAddress, Option<UntypedCapability>) {
        let mut target_untyped = Some(untyped);
        let mut unique = unsafe { Unique::new(0xffffffff_fffff000 as *mut _) };
        let mut p4: &mut PageTable<PageTableLevel4> = unsafe { unique.get_mut() };
        for i in 0..frame.count() {
            let virt_addr = dest_addr + i * PAGE_SIZE;
            let page = Page::new(virt_addr);
            let untyped = target_untyped.expect("Out of memory.");

            let (mut p3, untyped) = p4.next_table_create(page.p4_index(), untyped);
            let untyped = untyped.expect("Out of memory.");
            let (mut p2, untyped) = p3.next_table_create(page.p3_index(), untyped);
            let untyped = untyped.expect("Out of memory.");
            let (mut p1, untyped) = p2.next_table_create(page.p2_index(), untyped);

            assert!(p1[page.p1_index()].is_unused());

            p1[page.p1_index()].set_address(frame.block().start_addr() + i * PAGE_SIZE, frame.flags() | PRESENT);

            target_untyped = untyped;
        }
        (VirtualAddress { p4_addr: self.p4_block().start_addr(), addr: dest_addr }, target_untyped)
    }

    fn map_inactive(&self, frame: FrameCapability, dest_addr: usize, untyped: UntypedCapability)
                    -> (VirtualAddress, Option<UntypedCapability>) {
        use x86::tlb;

        let mut p4 = unsafe { &mut *(0xffffffff_fffff000 as *mut PageTable<PageTableLevel4>) };
        let flush_tlb = || unsafe { tlb::flush_all() };
        let mut target_untyped = Some(untyped);

        {
            let address511 = unsafe { controlregs::cr3() } as usize;
            let backup510 = p4[510].raw();

            p4[510].set_address(address511, PRESENT | WRITABLE);

            // overwrite recursive mapping
            p4[511].set_address(self.p4_block().start_addr(), PRESENT | WRITABLE);
            flush_tlb();

            // TODO Make sure the address doesn't refer to 511 or 510.
            for i in 0..frame.count() {
                let virt_addr = dest_addr + i * PAGE_SIZE;
                let page = Page::new(virt_addr);
                let untyped = target_untyped.expect("Out of memory.");

                let (mut p3, untyped) = p4.next_table_create(page.p4_index(), untyped);
                let untyped = untyped.expect("Out of memory.");
                let (mut p2, untyped) = p3.next_table_create(page.p3_index(), untyped);
                let untyped = untyped.expect("Out of memory.");
                let (mut p1, untyped) = p2.next_table_create(page.p2_index(), untyped);

                assert!(p1[page.p1_index()].is_unused());

                p1[page.p1_index()].set_address(frame.block().start_addr(), frame.flags() | PRESENT);

                target_untyped = untyped;
            }

            let mut original_p4 = unsafe { &mut *(0xffffff7f_bfdfe000 as *mut PageTable<PageTableLevel4>) };

            // restore recursive mapping to original p4 table
            original_p4[511].set_address(address511, PRESENT | WRITABLE);
            unsafe { original_p4[510].set_raw(backup510); }
            flush_tlb();
        }

        (VirtualAddress { p4_addr: self.p4_block().start_addr(), addr: dest_addr }, target_untyped)

    }

    pub fn identity_map(&self, frame: FrameCapability, untyped: UntypedCapability)
                        -> (VirtualAddress, Option<UntypedCapability>) {
        let dest_addr = frame.block().start_addr();
        self.map(frame, dest_addr, untyped)
    }

    pub unsafe fn switch_to(&self) {
        controlregs::cr3_write(self.p4_block().start_addr() as u64);
    }

    pub unsafe fn bootstrap(untyped: UntypedCapability)
                            -> (PageTableCapability, Option<UntypedCapability>) {
        let (block, remained) = UntypedCapability::retype(untyped, PAGE_SIZE, PAGE_SIZE);
        let mut table = unsafe { &mut *(block.start_addr() as *mut PageTable<PageTableLevel4>) };
        table.zero();
        table[511].set_address(block.start_addr(), PRESENT | WRITABLE);

        (PageTableCapability { p4_block: block }, remained)
    }

    pub fn from_untyped(untyped: UntypedCapability)
                        -> (PageTableCapability, Option<UntypedCapability>) {
        // TODO Use borrow_map to implement this.
        unimplemented!()
    }
}

impl Drop for PageTableCapability{
    fn drop(&mut self) {
        unimplemented!();
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

pub struct VirtualAddress {
    p4_addr: PhysicalAddress,
    addr: PhysicalAddress,
}

impl VirtualAddress {
    pub fn addr(&self) -> PhysicalAddress {
        self.addr
    }

    pub fn p4_addr(&self) -> PhysicalAddress {
        self.p4_addr
    }
}

struct Frame {
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
