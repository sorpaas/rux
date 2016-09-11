#[repr(packed)]
pub struct TaskStateSegment {
    reserved1: u32,
    pub sp0: u64,
    pub sp1: u64,
    pub sp2: u64,
    reserved2: u32,
    reserved3: u32,
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    reserved4: u32,
    reserved5: u32,
    reserved6: u16,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub const fn empty() -> TaskStateSegment {
        TaskStateSegment {
            reserved1: 0,
            reserved2: 0,
            reserved3: 0,
            reserved4: 0,
            reserved5: 0,
            reserved6: 0,
            sp0: 0,
            sp1: 0,
            sp2: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            iomap_base: 0,
        }
    }
}
