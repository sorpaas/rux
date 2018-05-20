// TODO Disable interrupt before entering those.
use core::mem::{size_of};
use util::{align_down, block_count};
use super::{PTEntry, PT_P, PT_RW, flush, BASE_PAGE_LENGTH};
use arch::init::{OBJECT_POOL_PT, OBJECT_POOL_START_VADDR};
use common::PAddr;

use core::ptr::NonNull;
use core::marker::{PhantomData, Unsize};
use core::ops::CoerceUnsized;
use core::fmt;

/// Represent a memory object, that converts a physical address to an
/// accessible object.
///
/// The struct is implemented using `ObjectPool`. When a new memory
/// object is created, a new entry on the `ObjectPool` PT is created,
/// thus makes it addressable. The entry is deleted once the memory
/// object struct is dropped.
///
/// # Safety
///
/// If you are going to wrap MemoryObject in any other struct, you
/// must make should that it is dropped last. However, Drop order in
/// Rust is currently undefined.
///
/// `ObjectGuard` requires T must be Sized.
pub struct MemoryObject<T: ?Sized> {
    paddr: PAddr,
    mapping_start_index: usize,
    mapping_size: usize,
    pointer: NonNull<T>,
    _marker: PhantomData<T>,
}

/// `ObjectGuard` pointers are not `Send` because the data they reference may be aliased.
impl<T: ?Sized> !Send for MemoryObject<T> { }

/// `ObjectGuard` pointers are not `Sync` because the data they reference may be aliased.
impl<T: ?Sized> !Sync for MemoryObject<T> { }

impl<T: ?Sized> MemoryObject<T> {
    /// Physical address of the memory object.
    pub fn paddr(&self) -> PAddr {
        self.paddr
    }

    /// Create a new memory object.
    ///
    /// # Safety
    ///
    /// PAddr must be a non-zero pointer.
    pub unsafe fn new(paddr: PAddr) -> Self where T: Sized {
        Self::slice(paddr, 1)
    }

    pub fn as_ptr(&self) -> *mut T {
        self.pointer.as_ptr()
    }

    pub unsafe fn as_ref(&self) -> &T {
        &*self.as_ptr()
    }

    pub unsafe fn as_mut(&mut self) -> &mut T {
        &mut *self.as_ptr()
    }

    /// Get a slice from the current memory object.
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
            flush(OBJECT_POOL_START_VADDR + (mapping_start_index * BASE_PAGE_LENGTH) + i * BASE_PAGE_LENGTH);
        }

        let vaddr = OBJECT_POOL_START_VADDR + ((mapping_start_index * BASE_PAGE_LENGTH) + before_start);

        MemoryObject::<T> {
            paddr: paddr,
            mapping_start_index: mapping_start_index,
            mapping_size: required_page_size,
            pointer: NonNull::new_unchecked(vaddr.into(): usize as *mut T),
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

impl<T: ?Sized> fmt::Pointer for MemoryObject<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.pointer.as_ptr(), f)
    }
}
