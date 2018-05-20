use arch::segmentation::{self, SegmentSelector};
use super::bit_field::BitField;
use super::{HandlerFunc, InterruptVector};

/// Interrupt descriptor table.
pub struct Idt([Entry; 256]);

/// An entry of the interrupt descriptor table.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Entry {
    pointer_low: u16,
    gdt_selector: SegmentSelector,
    options: BitField<u16>,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

/// Options in an entry of IDT.
pub struct EntryOptions<'a>(&'a mut Entry);

impl<'a> EntryOptions<'a> {
    /// Minimal settings of the entry.
    fn minimal(entry: &'a mut Entry) -> Self {
        let mut options = BitField::new(0);
        options.set_range(9..12, 0b111); // 'must-be-one' bits
        entry.options = options;
        EntryOptions(entry)
    }

    /// Create a new entry with default settings.
    fn new(entry: &'a mut Entry) -> Self {
        Self::minimal(entry)
            .set_present(true)
            .disable_interrupts(true)
            .set_stack_index(0x1)
    }

    /// Set the entry to be present.
    pub fn set_present(self, present: bool) -> Self {
        let mut options = self.0.options;
        options.set_bit(15, present);
        self.0.options = options;
        self
    }

    /// Disable interrupts when using this entry.
    pub fn disable_interrupts(self, disable: bool) -> Self {
        let mut options = self.0.options;
        options.set_bit(8, !disable);
        self.0.options = options;
        self
    }

    /// Set previlege level of this entry.
    pub fn set_privilege_level(self, dpl: u16) -> Self {
        let mut options = self.0.options;
        options.set_range(13..15, dpl);
        self.0.options = options;
        self
    }

    /// Set stack index to use in TSS for this interrupt entry.
    pub fn set_stack_index(self, index: u16) -> Self {
        let mut options = self.0.options;
        options.set_range(0..3, index);
        self.0.options = options;
        self
    }
}

impl Idt {
    /// Create a new IDT.
    pub fn new() -> Idt {
        Idt([Entry::missing(); 256])
    }

    /// Set an interrupt vector using a handler.
    pub fn set_handler(&mut self, entry: InterruptVector, handler: HandlerFunc)
        -> EntryOptions
    {
        self.0[entry as usize] = Entry::new(segmentation::cs(), handler);
        EntryOptions(&mut self.0[entry as usize])
    }

    /// Load this IDT.
    pub fn load(&self) {
        use super::dtables::{DescriptorTablePointer, lidt};
        use core::mem::size_of;

        let ptr = DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe { lidt(&ptr) };
    }
}

impl Entry {
    /// Create a new entry using the handler and GDT selector.
    fn new(gdt_selector: SegmentSelector, handler: HandlerFunc) -> Self {
        let pointer = handler as u64;
        let mut entry = Entry {
            gdt_selector: gdt_selector,
            pointer_low: pointer as u16,
            pointer_middle: (pointer >> 16) as u16,
            pointer_high: (pointer >> 32) as u32,
            options: BitField::new(0),
            reserved: 0,
        };
        EntryOptions::new(&mut entry);
        entry
    }

    /// Create a missing entry.
    fn missing() -> Self {
        let mut entry = Entry {
            gdt_selector: SegmentSelector::new(0),
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: BitField::new(0),
            reserved: 0,
        };
        EntryOptions::minimal(&mut entry);
        entry
    }
}
