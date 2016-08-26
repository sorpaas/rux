use common::{PAddr, VAddr};

#[derive(Debug)]
pub struct ArchInfo {
    memory_region_size: usize,
    memory_regions: [Option<MemoryRegion>; 16]
}

impl ArchInfo {
    pub fn memory_regions(&self) -> &[Option<MemoryRegion>; 16] {
        &self.memory_regions
    }

    pub fn push_memory_region(&mut self, region: MemoryRegion) {
        self.memory_regions[self.memory_region_size] = Some(region);
        self.memory_region_size += 1;
    }

    pub fn new() -> ArchInfo {
        ArchInfo { memory_region_size: 0,
                   memory_regions: [None; 16] }
    }
}

#[derive(Debug, Copy, Clone)]
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

    pub fn move_up(&mut self, npaddr: PAddr) {
        assert!(npaddr >= self.start_paddr);
        assert!(self.start_paddr + self.length > npaddr);
        let nlength = self.start_paddr.as_usize() + self.length - npaddr.as_usize();
        self.length = nlength;
        self.start_paddr = npaddr;
    }

    pub fn new(start_paddr: PAddr, length: usize) -> MemoryRegion {
        MemoryRegion {
            start_paddr: start_paddr,
            length: length
        }
    }
}
