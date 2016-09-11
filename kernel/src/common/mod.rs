pub use arch::{VAddr, PAddr};

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

    pub fn end_paddr(&self) -> PAddr {
        self.start_paddr + (self.length - 1)
    }

    pub fn skip_up(&mut self, region: &MemoryRegion) -> bool {
        if self.start_paddr() <= region.start_paddr() &&
            self.end_paddr() >= region.end_paddr()
        {
            self.move_up(region.start_paddr() + region.length());

            true
        } else {
            false
        }
    }

    pub fn move_up(&mut self, npaddr: PAddr) {
        assert!(npaddr >= self.start_paddr);
        assert!(self.start_paddr + self.length > npaddr);
        let nlength = self.start_paddr.into(): usize + self.length - npaddr.into(): usize;
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
