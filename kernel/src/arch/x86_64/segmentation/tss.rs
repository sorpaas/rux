/// Represents a Task State Segment. It holds the kernel stack
/// information used by interrupts.
#[repr(packed)]
#[allow(dead_code)]
pub struct TaskStateSegment {
    _reserved1: u32,
    pub sp0: u64,
    pub sp1: u64,
    pub sp2: u64,
    _reserved2: u32,
    _reserved3: u32,
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    _reserved4: u32,
    _reserved5: u32,
    _reserved6: u16,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    /// Create an empty TSS.
    pub const fn empty() -> TaskStateSegment {
        TaskStateSegment {
            _reserved1: 0,
            _reserved2: 0,
            _reserved3: 0,
            _reserved4: 0,
            _reserved5: 0,
            _reserved6: 0,
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
