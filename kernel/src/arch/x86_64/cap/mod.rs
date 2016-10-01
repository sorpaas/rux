mod paging;
mod thread;

pub use self::paging::{PageHalf, PML4Half, PDPTHalf, PDHalf, PTHalf};
pub use self::thread::{ThreadRuntime};

#[derive(Debug)]
pub enum ArchSpecificCapability {
    PDPT(PDPTHalf),
    PD(PDHalf),
    PT(PTHalf),
}
