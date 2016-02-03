use common::*;

pub fn necessary_page_counts(object_size: usize) -> usize {
    object_size / 1024 + 1
}

pub fn necessary_block_size(addr: PhysicalAddress, page_counts: usize) -> usize {
    let page_start_addr = necessary_page_start_addr(addr);
    let page_size = page_counts * PAGE_SIZE;

    (page_start_addr - addr) + page_size
}

pub fn necessary_page_start_addr(addr: PhysicalAddress) -> PhysicalAddress {
    addr + (PAGE_SIZE - addr % PAGE_SIZE)
}
