/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang) 
 *
 * arch/amd64/mod.rs
 * - Top-level file for amd64 architecture
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */

// Debug output channel (uses serial)
#[path = "debug.rs"]
pub mod debug;

mod paging;
mod init;
mod addr;
mod interrupt;
mod segmentation;

pub mod cap;
const KERNEL_BASE: u64 = 0xFFFFFFFF80000000;

extern {
    static kernel_end: u64;
}

fn kernel_start_paddr() -> PAddr {
    PAddr::from(0x100000: usize)
}

fn kernel_start_vaddr() -> VAddr {
    unsafe { kernel_paddr_to_vaddr(kernel_start_paddr()) }
}

fn kernel_end_paddr() -> PAddr {
    PAddr::from((&kernel_end as *const _) as u64 - KERNEL_BASE)
}

fn kernel_end_vaddr() -> VAddr {
    unsafe { kernel_paddr_to_vaddr(kernel_end_paddr()) }
}

unsafe fn kernel_paddr_to_vaddr(addr: PAddr) -> VAddr {
    VAddr::from(addr.into(): u64 + KERNEL_BASE)
}


#[cfg(any(target_arch = "x86_64"))]
pub unsafe fn outportb(port: u16, val: u8)
{
    asm!("outb %al, %dx" : : "{dx}"(port), "{al}"(val));
}

#[cfg(any(target_arch = "x86_64"))]
pub unsafe fn inportb(port: u16) -> u8
{
    let ret: u8;
    asm!("inb %dx, %al" : "={ax}"(ret): "{dx}"(port));
    ret
}

#[cfg(any(target_arch = "x86_64"))]
pub unsafe fn io_wait() {
    outportb(0x80, 0)
}

pub fn enable_timer() {
    interrupt::LOCAL_APIC.lock().enable_timer();
}

// Public interfaces
pub use self::paging::{MemoryObject};
pub use self::interrupt::{enable_interrupt, disable_interrupt, set_interrupt_handler,
                          Exception, TaskRuntime};
pub use self::init::{InitInfo};
// pub use self::cap::{ArchCap, PageHalf, PageFull};
pub use self::addr::{PAddr, VAddr};

// pub type TopPageTableHalf = self::cap::PML4Half;
// pub type TopPageTableFull = self::cap::PML4Full;
