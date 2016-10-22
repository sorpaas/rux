// TODO Disable interrupt before entering those.
use spin::{Mutex};
use core::mem::{size_of};
use core::slice;
use utils::{align_down, block_count};
use core::ptr::{Unique};
use super::{PTEntry, PT_P, PT_RW, flush, BASE_PAGE_LENGTH};
use arch::init::{OBJECT_POOL_PT, OBJECT_POOL_START_VADDR};
use common::{PAddr, VAddr};

use core::nonzero::{NonZero};
use core::marker::{PhantomData};
use core::ops::{Deref};
use core::mem;
use core::fmt;

static _next_free: Mutex<usize> = Mutex::new(0);

/// `ObjectGuard` requires T must be Sized.
pub struct ObjectGuard<T> {
    mapping_start_index: usize,
    mapping_size: usize,
    pointer: NonZero<*const T>,
    _marker: PhantomData<T>,
}

/// `ObjectGuard` pointers are not `Send` because the data they reference may be aliased.
impl<T> !Send for ObjectGuard<T> { }

/// `ObjectGuard` pointers are not `Sync` because the data they reference may be aliased.
impl<T> !Sync for ObjectGuard<T> { }

impl<T> ObjectGuard<T> {

    /// Safety: PAddr must be a non-zero pointer.
    pub unsafe fn new(paddr: PAddr) -> Self {
        let aligned = align_down(paddr, BASE_PAGE_LENGTH);
        let before_start = paddr.into(): usize - aligned.into(): usize;
        let size = size_of::<T>();
        let required_page_size = block_count((paddr + size).into(): usize - aligned.into(): usize,
                                             BASE_PAGE_LENGTH);

        let mut object_pool = OBJECT_POOL_PT.lock();
        let mapping_start_index: usize = {
            let mut mapping_start_index: Option<usize> = None;

            for i in 0..object_pool.len() {
                let mut available = true;
                for j in 0..required_page_size {
                    if object_pool[i + j].is_present() {
                        available = false;
                        break;
                    }
                }

                if available {
                    mapping_start_index = Some(i);
                    break;
                }
            }

            mapping_start_index
        }.unwrap();


        for i in 0..required_page_size {
            object_pool[mapping_start_index + i] = PTEntry::new(aligned + (i * BASE_PAGE_LENGTH), PT_P | PT_RW);
            unsafe { flush(OBJECT_POOL_START_VADDR + (mapping_start_index * BASE_PAGE_LENGTH) + i * BASE_PAGE_LENGTH); }
        }

        let vaddr = OBJECT_POOL_START_VADDR + ((mapping_start_index * BASE_PAGE_LENGTH) + before_start);

        ObjectGuard::<T> {
            mapping_start_index: mapping_start_index,
            mapping_size: required_page_size,
            pointer: NonZero::new(vaddr.into(): usize as *mut T),
            _marker: PhantomData
        }
    }
}

impl<T> Drop for ObjectGuard<T> {
    fn drop(&mut self) {
        let mut object_pool = OBJECT_POOL_PT.lock();

        for i in 0..self.mapping_size {
            object_pool[self.mapping_start_index + i] = PTEntry::empty();
            unsafe { flush(OBJECT_POOL_START_VADDR + (self.mapping_start_index * BASE_PAGE_LENGTH) + i * BASE_PAGE_LENGTH); }
        }
    }
}

impl<T> Deref for ObjectGuard<T> {
    type Target = *mut T;

    #[inline]
    fn deref(&self) -> &*mut T {
        unsafe { mem::transmute(&*self.pointer) }
    }
}

impl<T> fmt::Pointer for ObjectGuard<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&*self.pointer, f)
    }
}

pub unsafe fn with_object_vaddr<Return, F: FnOnce(VAddr) -> Return>(paddr: PAddr, size: usize, f: F)
                                                                -> Return {
    let aligned = align_down(paddr, BASE_PAGE_LENGTH);
    let before_start = paddr.into(): usize - aligned.into(): usize;
    let required_page_size = block_count((paddr + size).into(): usize - aligned.into(): usize,
                                         BASE_PAGE_LENGTH);

    let next_free_base;

    {
        let mut object_pool = OBJECT_POOL_PT.lock();
        let mut next_free = _next_free.lock();
        next_free_base = *next_free;

        for i in 0..required_page_size {
            object_pool[next_free_base + i] = PTEntry::new(aligned + (i * BASE_PAGE_LENGTH), PT_P | PT_RW);
            unsafe { flush(OBJECT_POOL_START_VADDR + (next_free_base * BASE_PAGE_LENGTH) + i * BASE_PAGE_LENGTH); }
        }

        *next_free = next_free_base + required_page_size;
    }

    let vaddr = OBJECT_POOL_START_VADDR + ((next_free_base * BASE_PAGE_LENGTH) + before_start);

    let result = f(vaddr);

    {
        let mut object_pool = OBJECT_POOL_PT.lock();
        let mut next_free = _next_free.lock();
        assert!(next_free_base == (*next_free - required_page_size));

        for i in 0..required_page_size {
            object_pool[next_free_base + i] = PTEntry::empty();
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
