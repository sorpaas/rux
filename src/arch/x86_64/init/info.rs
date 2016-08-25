use common::{PAddr, VAddr};

pub struct ArchInfo {
    free_memory_length: usize,
    free_memory_regions: [Option<MemoryRegion>; 16]
}

impl ArchInfo {
    pub fn free_memory_regions(&self) -> &[Option<MemoryRegion>; 16] {
        &self.free_memory_regions
    }
}

pub struct MemoryRegion {
    start_paddr: PAddr,
    length: usize
}

impl MemoryRegion {
    pub fn start_paddr(&self) -> PAddr {
        self.start_paddr
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn move_to(&mut self, npaddr: PAddr) {
        let nlength = self.start_paddr.as_usize() + self.length - npaddr.as_usize();
        self.length = nlength;
        self.start_paddr = npaddr;
    }
}
