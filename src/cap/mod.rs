use common::*;
use alloc::boxed::Box;
use core::mem::size_of;

enum Capability {
    // Memory resources capabilities, all has its start and end address, and a
    // next pointer to the next region (if available).
    //
    // A memory resources capability is essentially a pointer to a memory
    // location.
    UntypedMemoryCapability {
        start: PhysicalAddress,
        length: usize,
        next: Option<Box<Capability>>,
        first_child: Option<Box<Capability>>,
    },
    PageTableCapability {
        start: PhysicalAddress,
        length: usize,
        next: Option<Box<Capability>>,
    },
    CapabilityPoolCapability {
        start: PhysicalAddress,
        size: usize,
        next: Option<Box<Capability>>,
    },
    VirtualMemoryCapability {
        start: PhysicalAddress,
        size: usize,
        next: Option<Box<Capability>>,
    },

    // Hardware primitives.
    VGABuffer {
        start: PhysicalAddress,
        length: usize,
    },
}
