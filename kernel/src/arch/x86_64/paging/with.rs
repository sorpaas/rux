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
use core::marker::{PhantomData, Unsize};
use core::ops::{Deref, CoerceUnsized};
use core::mem;
use core::fmt;

/// CAUTION: If you are going to wrap MemoryObject in any other
/// struct, you must make should that it is dropped last. However,
/// Drop order in Rust is currently undefined.

/// `ObjectGuard` requires T must be Sized.
pub struct MemoryObject<T: ?Sized> {
    mapping_start_index: usize,
    mapping_size: usize,
    pointer: NonZero<*const T>,
    _marker: PhantomData<T>,
}

/// `ObjectGuard` pointers are not `Send` because the data they reference may be aliased.
impl<T: ?Sized> !Send for MemoryObject<T> { }

/// `ObjectGuard` pointers are not `Sync` because the data they reference may be aliased.
impl<T: ?Sized> !Sync for MemoryObject<T> { }

impl<T: ?Sized> MemoryObject<T> {
    /// Safety: PAddr must be a non-zero pointer.
    pub unsafe fn new(paddr: PAddr) -> Self where T: Sized {
        Self::slice(paddr, 1)
    }

    pub unsafe fn slice(paddr: PAddr, size: usize) -> Self where T: Sized {
        let aligned = align_down(paddr, BASE_PAGE_LENGTH);
        let before_start = paddr.into(): usize - aligned.into(): usize;
        let size = size_of::<T>() * size;
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

        MemoryObject::<T> {
            mapping_start_index: mapping_start_index,
            mapping_size: required_page_size,
            pointer: NonZero::new(vaddr.into(): usize as *mut T),
            _marker: PhantomData
        }
    }
}

impl<T: ?Sized, U: ?Sized> CoerceUnsized<MemoryObject<U>> for MemoryObject<T> where T: Unsize<U> { }

impl<T: ?Sized> Drop for MemoryObject<T> {
    fn drop(&mut self) {
        let mut object_pool = OBJECT_POOL_PT.lock();

        for i in 0..self.mapping_size {
            object_pool[self.mapping_start_index + i] = PTEntry::empty();
            unsafe { flush(OBJECT_POOL_START_VADDR + (self.mapping_start_index * BASE_PAGE_LENGTH) + i * BASE_PAGE_LENGTH); }
        }
    }
}

impl<T: ?Sized> Deref for MemoryObject<T> {
    type Target = *mut T;

    #[inline]
    fn deref(&self) -> &*mut T {
        unsafe { mem::transmute(&*self.pointer) }
    }
}

impl<T: ?Sized> fmt::Pointer for MemoryObject<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&*self.pointer, f)
    }
}
