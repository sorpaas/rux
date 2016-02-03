use common::*;
use core::mem::{align_of, replace, uninitialized, size_of};
use core::ops::Drop;
use core::ptr;
use alloc::boxed::Box;

use super::MemoryBlockCapability;
use super::untyped::UntypedMemoryCapability;

pub struct CapabilityPool {
    capabilities: [Option<CapabilityUnion>; CAPABILITY_POOL_COUNT],
    referred: bool,
}

impl PageBlockCapability for CapabilityPoolCapability {
    fn page_start_addr(&self) -> PhysicalAddress {
        self.page_start_addr
    }

    unsafe fn set_mapped_ptr(&mut self, ptr: Option<usize>) {
        self.ptr = ptr.and_then(|ptr| Some(ptr as *mut CapabilityPool))
    }

    unsafe fn set_mapped_p4_table_addr(&mut self, addr: Option<PhysicalAddress>) {
        self.mapped_p4_table_addr = addr
    }

    unsafe fn mapped_p4_table_addr(&self) {
        self.mapped_p4_table_addr
    }
}

impl MemoryBlockCapability for CapabilityPoolCapability {
    fn block_start_addr(&self) -> PhysicalAddress {
        self.block_start_addr
    }

    fn block_size(&self) -> usize {
        self.block_size
    }
}

impl Drop for CapabilityPoolCapability {
    fn drop(&mut self) {
        unimplemented!();
    }
}

impl CapabilityPoolCapability {
    pub fn from_untyped(cap: UntypedMemoryCapability)
                        -> (Option<CapabilityPoolCapability>, Option<UntypedMemoryCapability>) {
        let size = PAGE_SIZE;
        let align = PAGE_SIZE;
        let block_start_addr = cap.block_start_addr();
        let page_start_addr = cap.start_addr() + (align - cap.start_addr() % align);
        let page_end_addr = page_start_addr + size - 1;
        let block_size = page_end_addr - cap.start_addr() + 1;

        if page_end_addr > cap.end_addr() {
            (None, Some(cap))
        } else {
            cap.block_start_addr = cap.block_start_addr + block_size;
            cap.block_size = cap.block_size - block_size;

            let poolcap = CapabilityPoolCapability {
                block_start_addr: block_start_addr,
                block_size: block_size,
                page_start_addr: page_start_addr,
                mapped_p4_table_addr: None,
                ptr: None,
            };

            poolcap.active_identity_map();

            let pool = unsafe { &*self.ptr };
            pool.referred = false;

            for (i, element) in pool.iter_mut().enumerate() {
                let cap: Option<CapabilityUnion> = None;
                ptr::write(element, cap);
            }

            if cap.block_size() == 0 {
                (Some(pool_cap), None)
            } else {
                (Some(pool_cap), Some(cap))
            }
        }
    }
}
