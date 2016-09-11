// TODO Disable interrupt before entering those.
use spin::{Mutex};
use core::mem::{size_of};
use core::slice;
use utils::{align_down, block_count};
use core::ptr::{Unique};
use super::{PTEntry, PT_P, PT_RW, flush, BASE_PAGE_LENGTH};
use arch::init::{object_pool_pt, object_pool_pt_mut, OBJECT_POOL_SIZE, OBJECT_POOL_START_VADDR};
use common::{PAddr, VAddr};

static _next_free: Mutex<usize> = Mutex::new(0);

pub unsafe fn with_object_vaddr<Return, F: FnOnce(VAddr) -> Return>(paddr: PAddr, size: usize, f: F)
                                                                -> Return {
    let aligned = align_down(paddr, BASE_PAGE_LENGTH);
    let before_start = paddr.into(): usize - aligned.into(): usize;
    let required_page_size = block_count((paddr + size).into(): usize - aligned.into(): usize,
                                         BASE_PAGE_LENGTH);

    log!("required_page_size: {}", required_page_size);

    let next_free_base;

    {
        let mut next_free = _next_free.lock();
        next_free_base = *next_free;

        for i in 0..required_page_size {
            object_pool_pt_mut()[next_free_base + i] = PTEntry::new(aligned + (i * BASE_PAGE_LENGTH), PT_P | PT_RW);
            unsafe { flush(OBJECT_POOL_START_VADDR + (next_free_base * BASE_PAGE_LENGTH) + i * BASE_PAGE_LENGTH); }
        }

        *next_free = next_free_base + required_page_size;
    }

    let vaddr = OBJECT_POOL_START_VADDR + ((next_free_base * BASE_PAGE_LENGTH) + before_start);

    let result = f(vaddr);

    {
        let mut next_free = _next_free.lock();
        assert!(next_free_base == (*next_free - required_page_size));

        for i in 0..required_page_size {
            object_pool_pt_mut()[next_free_base + i] = PTEntry::empty();
            unsafe { flush(OBJECT_POOL_START_VADDR + (next_free_base * BASE_PAGE_LENGTH) + i * BASE_PAGE_LENGTH); }
        }

        *next_free = next_free_base;
    }

    result
}

pub unsafe fn with_object_unique<T, Return, F: FnOnce(Unique<T>) -> Return>(paddr: PAddr, f: F)
                                                                            -> Return {
    let size = size_of::<T>();
    with_object_vaddr(paddr, size, |vaddr| {
        let unique = unsafe {
            Unique::new(vaddr.into(): usize as *mut T) };
        f(unique)
    })
}
    
pub unsafe fn with_slice<T, Return, F: FnOnce(&[T]) -> Return>(paddr: PAddr, size: usize, f: F)
                                                               -> Return {
    let allsize = size * (size_of::<T>());
    with_object_vaddr(paddr, allsize, |vaddr| {
        let slice = unsafe {
            slice::from_raw_parts::<T>(vaddr.into(): usize as *const _, allsize) };
        f(slice)
    })
}

pub unsafe fn with_slice_mut<T, Return, F: FnOnce(&mut [T]) -> Return>(paddr: PAddr, size: usize, f: F)
                                                                       -> Return {
    let allsize = size * (size_of::<T>());
    with_object_vaddr(paddr, allsize, |vaddr| {
        let mut slice = unsafe {
            slice::from_raw_parts_mut::<T>(vaddr.into(): usize as *mut _, allsize) };
        f(slice)
    })
}

pub unsafe fn with_object<T, Return, F: FnOnce(&T) -> Return>(paddr: PAddr, f: F) -> Return {
    with_object_unique(paddr, |unique| {
        f(unsafe { unique.get() })
    })
}

pub unsafe fn with_object_mut<T, Return, F: FnOnce(&mut T) -> Return>(paddr: PAddr, f: F) -> Return {
    with_object_unique(paddr, |mut unique| {
        f(unsafe { unique.get_mut() })
    })
}
