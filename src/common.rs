pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub enum PageTableType {
    PageMapLevel4,
    PageDirectoryPointer,
    PageDirectory,
    PageTable,
}
