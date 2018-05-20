use common::*;
use arch::init::{LOCAL_APIC_PAGE_VADDR, IO_APIC_PAGE_VADDR};
use util::{Mutex};
use super::{InterruptVector};

/// Local APIC pointer.
#[derive(Debug)]
pub struct LocalAPIC {
    address: VAddr,
}

/// I/O APIC pointer.
#[derive(Debug)]
pub struct IOAPIC {
    address: VAddr,
}

/// The local APIC static.
pub static LOCAL_APIC: Mutex<LocalAPIC> = Mutex::new(LocalAPIC {
    address: LOCAL_APIC_PAGE_VADDR
});

/// The I/O APIC static.
pub static IO_APIC: Mutex<IOAPIC> = Mutex::new(IOAPIC {
    address: IO_APIC_PAGE_VADDR
});

#[allow(dead_code)]
impl LocalAPIC {
    /// Read a value from the local APIC.
    ///
    /// # Safety
    ///
    /// `reg` must be valid.
    unsafe fn read(&self, reg: u32) -> u32 {
        use core::intrinsics::{volatile_load};
        volatile_load((self.address.into(): usize + reg as usize) as *const u32)
    }

    /// Write a value to the local APIC.
    ///
    /// # Safety
    ///
    /// `reg` must be valid.
    unsafe fn write(&mut self, reg: u32, value: u32) {
        use core::intrinsics::{volatile_store};
        volatile_store((self.address.into(): usize + reg as usize) as *mut u32, value);
    }

    /// APIC id.
    pub fn id(&self) -> u32 {
        unsafe { self.read(0x20) }
    }

    /// APIC version.
    pub fn version(&self) -> u32 {
        unsafe { self.read(0x30) }
    }

    /// Spurious interrupt vector.
    pub fn siv(&self) -> u32 {
        unsafe { self.read(0xF0) }
    }

    /// Set the spurious interrupt vector.
    pub fn set_siv(&mut self, value: u32) {
        unsafe { self.write(0xF0, value) }
    }

    /// Send End of Interrupt.
    pub fn eoi(&mut self) {
        unsafe { self.write(0xB0, 0) }
    }

    /// Enable timer with a specific value.
    pub fn enable_timer(&mut self) {
        unsafe {
            self.write(0x3E0, 0x3);
            self.write(0x380, 0x10000);
            self.write(0x320, (1<<17) | 0x40);
            log!("timer register is 0b{:b}", self.read(0x320));
        }
    }

    /// Current error status.
    pub fn error_status(&self) -> u32 {
        unsafe { self.read(0x280) }
    }
}

#[allow(dead_code)]
impl IOAPIC {
    /// Read a value from the I/O APIC.
    ///
    /// # Safety
    ///
    /// `reg` must be valid.
    unsafe fn read(&self, reg: u32) -> u32 {
        use core::intrinsics::{volatile_load, volatile_store};
        volatile_store((self.address.into(): usize + 0x0 as usize) as *mut u32, reg);
        volatile_load((self.address.into(): usize + 0x10 as usize) as *const u32)
    }

    /// Write a value to the I/O APIC.
    ///
    /// # Safety
    ///
    /// `reg` must be valid.
    unsafe fn write(&mut self, reg: u32, value: u32) {
        use core::intrinsics::volatile_store;
        volatile_store((self.address.into(): usize + 0x0 as usize) as *mut u32, reg);
        volatile_store((self.address.into(): usize + 0x10 as usize) as *mut u32, value);
    }

    /// I/O APIC id.
    pub fn id(&self) -> u32 {
        unsafe { self.read(0x0) }
    }

    /// I/O APIC version.
    pub fn version(&self) -> u32 {
        unsafe { self.read(0x1) }
    }

    /// I/O APIC arbitration id.
    pub fn arbitration_id(&self) -> u32 {
        unsafe { self.read(0x2) }
    }

    /// Set IRQ to an interrupt vector.
    pub fn set_irq(&mut self, irq: u8, apic_id: u8, vector: InterruptVector) {
        let vector = vector as u8;

        let low_index: u32 = 0x10 + (irq as u32) * 2;
        let high_index: u32 = 0x10 + (irq as u32) * 2 + 1;

        let mut high = unsafe { self.read(high_index) };
        high &= !0xff000000;
        high |= (apic_id as u32) << 24;
        unsafe { self.write(high_index, high) };

        let mut low = unsafe { self.read(low_index) };
        low &= !(1<<16);
        low &= !(1<<11);
        low &= !0x700;
        low &= !0xff;
        low |= vector as u32;
        unsafe { self.write(low_index, low) };
    }
}
