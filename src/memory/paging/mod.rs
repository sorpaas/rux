mod entry;
mod table;

use memory::PAGE_SIZE;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub fn translate(address: VirtualAddress) -> Option<PhysicalAddress> {
    let p4 = unsafe { &*table::P4 };

    p4.next_table(p4_index(address))
        .and_then(|p3| p3.next_table(p3_index(address)))
        .and_then(|p2| p2.next_table(p2_index(address)))
        .and_then(|p1| p1[p1_index(address)].address())
        .and_then(|base_addr| Some(base_addr + offset(address)))
}

pub fn p4_index(address: VirtualAddress) -> usize {
    (address >> 39) & 0o777
}

pub fn p3_index(address: VirtualAddress) -> usize {
    (address >> 30) & 0o777
}

pub fn p2_index(address: VirtualAddress) -> usize {
    (address >> 21) & 0o777
}

pub fn p1_index(address: VirtualAddress) -> usize {
    (address >> 12) & 0o777
}

pub fn offset(address: VirtualAddress) -> usize {
    (address >> 0) & 0o7777
}
