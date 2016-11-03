use arch::{KERNEL_BASE};
use arch::segmentation::{SegmentDescriptor, SegmentSelector, TaskStateSegment};
use common::*;
use core::mem::{size_of};

extern {
    static mut GDT: [SegmentDescriptor; 9];
    static init_stack: u64;
}

static mut TSS: TaskStateSegment = TaskStateSegment::empty();

/// Load the task state register.
pub unsafe fn load_tr(sel: SegmentSelector) {
    asm!("ltr $0" :: "r" (sel));
}

pub unsafe fn set_kernel_stack(addr: u64) {
    TSS.sp0 = addr;
}

pub fn init() {
    unsafe {
        use arch::segmentation::{DESC_P, DESC_L, DESC_AVL, DESC_DPL3,
                                 TYPE_SYS_TSS_AVAILABLE,
                                 TYPE_C_ER};
        let kernel_stack = &init_stack as *const _ as u64;
        let tss_vaddr = &TSS as *const _ as u64;

        set_kernel_stack(kernel_stack);
        GDT[7] = SegmentDescriptor::new((tss_vaddr & 0xFFFFFFFF) as u32,
                                        size_of::<TaskStateSegment>() as u32);
        GDT[7].insert(DESC_P | TYPE_SYS_TSS_AVAILABLE | DESC_DPL3);
        GDT[8] = SegmentDescriptor::from_raw(tss_vaddr >> 32);

        log!("kernel_stack = 0x{:x}", kernel_stack);
        // asm!("ltr ax" :: "{rax}"(&GDT[7] as *const _ as usize)
        //      : "rax" : "intel", "volatile");
        load_tr(SegmentSelector::new(7));
    }
}
