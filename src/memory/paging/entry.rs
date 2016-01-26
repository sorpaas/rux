use memory::paging::{PhysicalAddress, VirtualAddress};
use memory::Frame;
use multiboot2::ElfSection;

pub struct Entry(u64);

impl Entry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn physical_address(&self) -> Option<PhysicalAddress> {
        if self.flags().contains(PRESENT) {
            Some(self.0 as usize & 0x000fffff_fffff000)
        } else {
            None
        }
    }

    pub fn pointed_frame(&self) -> Option<Frame> {
        self.physical_address()
            .and_then(|addr| Some(Frame::containing_address(addr)))
    }

    pub fn set_address(&mut self, address: PhysicalAddress, flags: EntryFlags) {
        assert!(address & !0x000fffff_fffff000 == 0);
        self.0 = (address as u64) | flags.bits();
    }

    pub fn set_frame(&mut self, frame: Frame, flags: EntryFlags) {
        self.set_address(frame.start_address(), flags)
    }
}

bitflags! {
    flags EntryFlags: u64 {
        const PRESENT =         1 << 0,
        const WRITABLE =        1 << 1,
        const USER_ACCESSIBLE = 1 << 2,
        const WRITE_THROUGH =   1 << 3,
        const NO_CACHE =        1 << 4,
        const ACCESSED =        1 << 5,
        const DIRTY =           1 << 6,
        const HUGE_PAGE =       1 << 7,
        const GLOBAL =          1 << 8,
        const NO_EXECUTE =      1 << 63,
    }
}

impl EntryFlags {
    pub fn from_elf_section_flags(section: &ElfSection) -> EntryFlags {
        use multiboot2::{ELF_SECTION_ALLOCATED, ELF_SECTION_WRITABLE,
                         ELF_SECTION_EXECUTABLE};

        let mut flags = EntryFlags::empty();

        if section.flags().contains(ELF_SECTION_ALLOCATED) {
            // section is loaded to memory
            flags = flags | PRESENT;
        }
        if section.flags().contains(ELF_SECTION_WRITABLE) {
            flags = flags | WRITABLE;
        }
        if !section.flags().contains(ELF_SECTION_EXECUTABLE) {
            flags = flags | NO_EXECUTE;
        }

        flags
    }
}
