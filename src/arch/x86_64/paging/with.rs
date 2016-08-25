// TODO Disable interrupt before entering those.
use spin::{Mutex};
use core::mem::{size_of};
use utils::{align_down, block_count};
use core::ptr::{Unique};
use super::{PTEntry, PT_P, PT_RW, flush, BASE_PAGE_LENGTH};
use arch::{object_pool_pt, object_pool_pt_mut, OBJECT_POOL_SIZE,
            OBJECT_POOL_START_VADDR};
use common::{PAddr, VAddr};

static _next_free: Mutex<usize> = Mutex::new(0);

fn with_object_unique<T, Return, F: Fn(Unique<T>) -> Return>(paddr: PAddr, f: F) -> Return {
    let size = size_of::<T>();
    let aligned = align_down(paddr, BASE_PAGE_LENGTH);
    let before_start = paddr.as_usize() - aligned.as_usize();
    let required_page_size = block_count((paddr + size).as_usize() - aligned.as_usize(), BASE_PAGE_LENGTH);

    log!("required_page_size: {}", required_page_size);

    let next_free_base;

    {
        let mut next_free = _next_free.lock();
        next_free_base = *next_free;

        for i in 0..required_page_size {
            object_pool_pt_mut()[next_free_base + i] = PTEntry::new(aligned + (i * BASE_PAGE_LENGTH), PT_P | PT_RW);
        }

        *next_free = next_free_base + required_page_size;
    }

    let vaddr = OBJECT_POOL_START_VADDR + ((next_free_base * BASE_PAGE_LENGTH) + before_start);
    let unique = unsafe { Unique::new(vaddr.as_usize() as *mut T) };

    let result = f(unique);

    {
        let mut next_free = _next_free.lock();
        assert!(next_free_base == (*next_free - required_page_size));

        for i in 0..required_page_size {
            object_pool_pt_mut()[next_free_base + i] = PTEntry::empty();
        }

        *next_free = next_free_base;
    }

    result
}

pub fn with_object<T, Return, F: Fn(&T) -> Return>(paddr: PAddr, f: F) -> Return {
    with_object_unique(paddr, |unique| {
        f(unsafe { unique.get() })
    })
}

pub fn with_object_mut<T, Return, F: Fn(&mut T) -> Return>(paddr: PAddr, f: F) -> Return {
    with_object_unique(paddr, |mut unique| {
        f(unsafe { unique.get_mut() })
    })
}
