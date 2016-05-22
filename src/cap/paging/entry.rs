use common::*;

pub struct PageTableEntry(u64);

impl PageTableEntry {
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

    pub fn set_address(&mut self, address: PhysicalAddress, flags: EntryFlags) {
        assert!(address & !0x000fffff_fffff000 == 0);
        self.0 = (address as u64) | flags.bits();
    }

    pub fn raw(&self) -> u64 {
        self.0
    }

    pub unsafe fn set_raw(&mut self, raw: u64) {
        self.0 = raw;
    }
}

