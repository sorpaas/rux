use cap::{CPoolHalf, UntypedHalf, PageHalf};
use super::*;

use core::mem::{size_of};
use core::slice;
use core::ptr::{Unique};
use core::ops::{Add, AddAssign};

pub trait ArchTrait where Self: Sized {
    type VAddr: Copy + Clone + Eq + Ord + PartialEq + PartialOrd +
        From<usize> + Into<usize> + From<u64> + Into<u64> +
        From<u32> + Into<u32> + Add<usize, Output=VAddr> +
        AddAssign<usize>;
    type PAddr: Copy + Clone + Eq + Ord + PartialEq + PartialOrd +
        From<usize> + Into<usize> + From<u64> + Into<u64> +
        From<u32> + Into<u32> + Add<usize, Output=PAddr> +
        AddAssign<usize>;
    type InitInfo: InitInfoTrait;
    type InterruptInfo: InterruptInfoTrait;
    type TopPageTableHalf: TopPageTableHalfTrait;
    type PageHalf: PageHalfTrait;
    type ArchSpecificCapability;

    // This may cause data race, so it is unsafe.
    unsafe fn with_object_vaddr<Return, F: FnOnce(VAddr) -> Return>(paddr: PAddr, size: usize, f: F)
                                                                    -> Return;

    unsafe fn with_object_unique<T, Return, F: FnOnce(Unique<T>) -> Return>(paddr: PAddr, f: F)
                                                                            -> Return {
        let size = size_of::<T>();
        Self::with_object_vaddr(paddr, size, |vaddr| {
            let unique = unsafe {
                Unique::new(vaddr.as_usize() as *mut T) };
            f(unique)
        })
    }
    
    unsafe fn with_slice<T, Return, F: FnOnce(&[T]) -> Return>(paddr: PAddr, size: usize, f: F)
                                                               -> Return {
        let allsize = size * (size_of::<T>());
        Self::with_object_vaddr(paddr, allsize, |vaddr| {
            let slice = unsafe {
                slice::from_raw_parts::<T>(vaddr.as_usize() as *const _, allsize) };
            f(slice)
        })
    }

    unsafe fn with_slice_mut<T, Return, F: FnOnce(&mut [T]) -> Return>(paddr: PAddr, size: usize, f: F)
                                                                       -> Return {
        let allsize = size * (size_of::<T>());
        Self::with_object_vaddr(paddr, allsize, |vaddr| {
            let mut slice = unsafe {
                slice::from_raw_parts_mut::<T>(vaddr.as_usize() as *mut _, allsize) };
            f(slice)
        })
    }

    unsafe fn with_object<T, Return, F: FnOnce(&T) -> Return>(paddr: PAddr, f: F) -> Return {
        Self::with_object_unique(paddr, |unique| {
            f(unsafe { unique.get() })
        })
    }

    unsafe fn with_object_mut<T, Return, F: FnOnce(&mut T) -> Return>(paddr: PAddr, f: F) -> Return {
        Self::with_object_unique(paddr, |mut unique| {
            f(unsafe { unique.get_mut() })
        })
    }

    fn enable_interrupt();
    fn disable_interrupt();

    fn set_interrupt_handler(handler: fn(info: Self::InterruptInfo));
    unsafe fn switch_to_user_mode(code_vaddr: VAddr, stack_vaddr: VAddr);
}

pub trait InitInfoTrait {
    fn free_regions(&self) -> Iterator<Item=MemoryRegion>;
    fn kernel_region(&self) -> MemoryRegion;
    fn rinit_region(&self) -> MemoryRegion;
}

pub trait InterruptInfoTrait {

}

pub trait TopPageTableHalfTrait {
    fn new(untyped: &mut UntypedHalf) -> Self;

    // This takes untyped, and thus should only be called once in kmain.
    fn map(&mut self, vaddr: VAddr, page: &PageHalf, untyped: &mut UntypedHalf, cpool: &mut CPoolHalf);
    
    // Unsafe due to memory distortion.
    unsafe fn switch_to(&self);

    const fn length() -> usize;
}

pub trait PageHalfTrait {
    fn new(untyped: &mut UntypedHalf) -> Self;
    const fn length() -> usize;
}
