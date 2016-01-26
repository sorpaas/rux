mod entry;
mod table;
mod temporary_page;
mod mapper;

use multiboot2::BootInformation;
use memory::{PAGE_SIZE, Frame, FrameAllocator};
use core::ptr::Unique;
use core::ops::{Deref, DerefMut};

use self::entry::{EntryFlags};
use self::table::{P4, Table, Level4};
use self::temporary_page::{TemporaryPage};

pub use self::mapper::Mapper;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub const ENTRY_COUNT: usize = 512;

#[derive(Debug, Clone, Copy)]
pub struct Page {
    number: usize,
}

pub fn remap_kernel<A>(allocator: &mut A,
                       boot_info: &BootInformation)
    where A: FrameAllocator {
    use core::ops::Range;

    let mut temporary_page = TemporaryPage::new(Page { number: 0xcafebabe },
                                            allocator);

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("no more frames");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        use self::entry::WRITABLE;

        let elf_sections_tag = boot_info.elf_sections_tag()
            .expect("Memory map tag required");

        for section in elf_sections_tag.sections() {
            use multiboot2::ELF_SECTION_ALLOCATED;

            if !section.flags().contains(ELF_SECTION_ALLOCATED) {
                // section is not loaded to memory
                continue;
            }

            println!("mapping section at addr: {:#x}, size: {:#x}",
                     section.addr, section.size);

            let flags = EntryFlags::from_elf_section_flags(section);

            let range = Range {
                start: section.addr as usize,
                end: (section.addr + section.size) as usize,
            };

            for address in range.step_by(PAGE_SIZE) {
                assert!(address % PAGE_SIZE == 0,
                        "sections need to be page aligned");
                let frame = Frame::containing_address(address);
                mapper.identity_map(frame, flags, allocator);
            }
        }

        let vga_buffer_frame = Frame::containing_address(0xb8000);
        mapper.identity_map(vga_buffer_frame, WRITABLE, allocator);

    });

    let old_table = active_table.switch(new_table);
    println!("New table!");

    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page, allocator);
    println!("guard page at {:#x}", old_p4_page.start_address());
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn with<F>(&mut self,
                   table: &mut InactivePageTable,
                   temporary_page: &mut temporary_page::TemporaryPage,
                   f: F)
        where F: FnOnce(&mut Mapper) {
        use x86::{controlregs, tlb};
        let flush_tlb = || unsafe { tlb::flush_all() };

        {
            use self::entry::{PRESENT, WRITABLE};

            let backup = Frame::containing_address(unsafe { controlregs::cr3() } as usize);

            // map temporary_page to current p4 table
            let p4_table = temporary_page.map_table_frame(backup.clone(), self);

            // overwrite recursive mapping
            self.p4_mut()[511].set_frame(table.p4_frame.clone(), PRESENT | WRITABLE);
            flush_tlb();

            // execute f in the new context
            f(self);

            // restore recursive mapping to original p4 table
            p4_table[511].set_frame(backup, PRESENT | WRITABLE);
            flush_tlb();
        }

        temporary_page.unmap(self);
    }

    pub fn switch(&mut self,
                  new_table: InactivePageTable)
                  -> InactivePageTable {
        use x86::controlregs;

        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(unsafe { controlregs::cr3() } as usize),
        };
        unsafe {
            controlregs::cr3_write(new_table.p4_frame.start_address() as u64);
        }
        old_table
    }
}

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame,
               active_table: &mut ActivePageTable,
               temporary_page: &mut TemporaryPage)
               -> InactivePageTable {
        {
            use self::entry::{PRESENT, WRITABLE};

            let table = temporary_page.map_table_frame(frame.clone(), active_table);
            // now we are able to zero the table
            table.zero();
            // set up recursive mapping for the table
            table[511].set_frame(frame.clone(), PRESENT | WRITABLE);
        }
        temporary_page.unmap(active_table);

        InactivePageTable { p4_frame: frame }
    }
}

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Page {
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

pub fn test_paging<A>(allocator: &mut A)
    where A: FrameAllocator {
    let mut page_table = unsafe { ActivePageTable::new() };

    // address 0 is mapped
    println!("Some = {:?}", page_table.translate(0));
    // second P1 entry
    println!("Some = {:?}", page_table.translate(4096));
    // second P2 entry
    println!("Some = {:?}", page_table.translate(512 * 4096));
    // 300th P2 entry
    println!("Some = {:?}", page_table.translate(300 * 512 * 4096));
    // second P3 entry
    println!("None = {:?}", page_table.translate(512 * 512 * 4096));
    // last mapped byte
    println!("Some = {:?}", page_table.translate(512 * 512 * 4096 - 1));

    let addr = 42 * 512 * 512 * 4096; // 42th P3 entry
    let page = Page::containing_address(addr);
    let frame = allocator.allocate_frame().expect("no more frames");
    println!("None = {:?}, map to {:?}",
             page_table.translate(addr),
             frame);
    page_table.map_to(page, frame, EntryFlags::empty(), allocator);
    println!("Some = {:?}", page_table.translate(addr));
    println!("next free frame: {:?}", allocator.allocate_frame());

    page_table.unmap(Page::containing_address(addr), allocator);
    println!("None = {:?}", page_table.translate(addr));

    println!("{:#x}", unsafe {
        *(Page::containing_address(addr).start_address() as *const u64)
    });

}
