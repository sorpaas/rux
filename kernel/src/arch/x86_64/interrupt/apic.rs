use common::*;
use arch::init::{LOCAL_APIC_PAGE_VADDR, IO_APIC_PAGE_VADDR};

pub struct LocalAPIC {
    address: VAddr,
}

pub struct IOAPIC {
    address: VAddr,
}

pub static LOCAL_APIC: LocalAPIC = LocalAPIC {
    address: LOCAL_APIC_PAGE_VADDR
};

pub static IO_APIC: IOAPIC = IOAPIC {
    address: IO_APIC_PAGE_VADDR
};

impl LocalAPIC {
    unsafe fn read(&self, reg: u32) -> u32 {
        use core::intrinsics::{volatile_load};
        volatile_load((self.address.into(): usize + reg as usize) as *const u32)
    }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        use core::intrinsics::{volatile_store};
        volatile_store((self.address.into(): usize + reg as usize) as *mut u32, value);
    }

    pub fn id(&self) -> u32 {
        unsafe { self.read(0x20) }
    }

    pub fn version(&self) -> u32 {
        unsafe { self.read(0x30) }
    }
}

impl IOAPIC {
    unsafe fn read(&self, reg: u32) -> u32 {
        use core::intrinsics::{volatile_load};
        volatile_load((self.address.into(): usize + reg as usize) as *const u32)
    }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        use core::intrinsics::{volatile_store};
        volatile_store((self.address.into(): usize + reg as usize) as *mut u32, value);
    }
}
