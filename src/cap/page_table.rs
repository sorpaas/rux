// You can only access active page table using the recursive trick. Other page
// tables need to be temporarily mapped in order to be accessible.

pub struct PageTableCapability {
    start: PhysicalAddress,
    physical_start: PhysicalAddress,
}
