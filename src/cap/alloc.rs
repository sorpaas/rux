use common::*;
use core::marker::PhantomData;

/// This is the implementation of Unique for Rux.
///
/// Unlike in the Rust library, the Unique doesn't actually de-allocate the
/// underlying box value. Instead, that is managed by different capabilities.

pub struct AddressCapability<T> {
    addr: VirtualAddress,
    page_table_addr: PhysicalAddress,
    _marker: PhantomData<T>,
}

impl<T> AddressCapability<T> {
    pub unsafe fn new(addr: VirtualAddress, page_table_addr: PhysicalAddress) -> Self {
        AddressCapability::<T> {
            addr: addr,
            page_table_addr: page_table_addr,
            _marker: PhantomData,
        }
    }

    pub fn is_active(&self) -> bool {
        unsafe { ::x86::controlregs::cr3() as PhysicalAddress == self.page_table_addr }
    }

    pub fn get(&self) -> Unique<T> {
        assert!(self.is_active());

        Unique::<T>::new(unsafe { &mut *(self.addr as *mut _) })
    }
}

impl<T> Drop for AddressCapability<T> {
    fn drop(&mut self) {
        unimplemented!()
    }
}

pub struct UniqueBox<T> {
    value: T,
    referred: bool,
}

pub struct Unique<T> {
    _ptr: *mut UniqueBox<T>,
}

impl<T> Unique<T> {
    pub fn new(ptr: *mut UniqueBox<T>) -> Unique<T> {
        let mut unique = Unique { _ptr: ptr };
        {
            let value = unique.borrow_box_mut();

            assert!(value.referred == false);
            value.referred = true;
        }
        unique
    }

    pub fn borrow_box<'r>(&'r self) -> &'r UniqueBox<T> {
        unsafe { &*self._ptr }
    }

    pub fn borrow<'r>(&'r self) -> &'r T {
        &(self.borrow_box().value)
    }

    pub fn borrow_box_mut<'r>(&'r mut self) -> &'r mut UniqueBox<T> {
        unsafe { &mut *self._ptr }
    }

    pub fn borrow_mut<'r>(&'r mut self) -> &'r mut T {
        &mut (self.borrow_box_mut().value)
    }
}

impl<T> Drop for Unique<T> {
    fn drop(&mut self) {
        self.borrow_box_mut().referred = false;
    }
}
