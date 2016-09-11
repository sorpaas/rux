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

// x86 port IO 
#[path = "io.rs"]
mod x86_io;

// Debug output channel (uses serial)
#[path = "debug.rs"]
pub mod debug;

mod paging;
mod init;
mod addr;
mod interrupt;
mod segmentation;
mod cap;

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


// Public interfaces
pub use self::paging::{with_object_vaddr, with_object_unique,
                       with_slice, with_slice_mut,
                       with_object, with_object_mut};
pub use self::interrupt::{enable_interrupt, disable_interrupt, set_interrupt_handler,
                          InterruptInfo};
pub use self::init::{InitInfo};
pub use self::cap::{ArchSpecificCapability, PageHalf};
pub use self::addr::{PAddr, VAddr};

pub type TopPageTableHalf = self::cap::PML4Half;

pub unsafe fn switch_to_user_mode(code_vaddr: VAddr, stack_vaddr: VAddr) {
    unsafe {
        let stack_addr: usize = stack_vaddr.into();
        let code_start: usize = code_vaddr.into();
        let code_seg = 0x28 | 0x3;
        let data_seg = 0x30 | 0x3;
        asm!("mov ds, rax
              mov es, rax
              mov fs, rax
              mov gs, rax

              push rax
              push rbx
              pushfq
              push rcx
              push rdx
              iretq"
             :: "{rax}"(data_seg), "{rbx}"(stack_vaddr), "{rcx}"(code_seg), "{rdx}"(code_start)
             : "memory" : "intel", "volatile");
    }
}
