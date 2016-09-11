use common::{PAddr, VAddr};

pub fn align_up(paddr: PAddr, alignment: usize) -> PAddr {
    let raw = paddr.into(): usize;
    let aligned = if raw % alignment == 0 {
        raw
    } else {
        raw + (alignment - (raw % alignment))
    };
    PAddr::from(aligned)
}

pub fn align_down(paddr: PAddr, alignment: usize) -> PAddr {
    let raw = paddr.into(): usize;
    let aligned = if raw % alignment == 0 {
        raw
    } else {
        raw - (raw % alignment)
    };
    PAddr::from(aligned)
}

pub fn block_count(length: usize, block_length: usize) -> usize {
    if length % block_length == 0 {
        length / block_length
    } else {
        length / block_length + 1
    }
}
