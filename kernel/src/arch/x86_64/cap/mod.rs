mod paging;

pub use self::paging::{PageHalf, PML4Half, PDPTHalf, PDHalf, PTHalf};

#[derive(Debug)]
pub enum ArchSpecificCapability {
    PDPT(PDPTHalf),
    PD(PDHalf),
    PT(PTHalf),
}
