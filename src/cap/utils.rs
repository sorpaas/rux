use common::*;

pub fn align(addr: PhysicalAddress, alignment: usize) -> PhysicalAddress {
    if addr % alignment == 0 {
        addr
    } else {
        addr + (alignment - addr % alignment)
    }
}
