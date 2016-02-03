
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
    pub unsafe fn new(addr: VirtualdAdress, page_table_addr: PhysicalAddress) -> Self {
        AddressCapability<T> {
            addr: addr,
            page_table_addr: page_table_addr,
            _marker: PhantomData,
        }
    }

    pub fn is_active(&self) -> bool {
        unsafe { controlregs::cr3() as PhysicalAddress == self.page_table_addr }
    }

    pub fn get(&self) -> Unique<T> {
        assert!("A gettable address capability must be actively mapped.",
                self.is_active());

        Unique<T>::new(unsafe { &mut *(self.addr as *mut _) })
    }
}

impl<T> Drop for AddressCapability<T> {
    fn drop(&self) {
        unimplemented!()
    }
}

pub struct UniqueBox<T: ?Sized> {
    value: T,
    referred: bool,
}

pub struct Unique<T: ?Sized> {
    _ptr: *mut UniqueBox<T>,
}

impl<T: ?Sized> Unique<T> {
    pub fn new(ptr: *mut UniqueBox<T>) -> Unique<T> {
        let unique = Unique { _ptr: ptr };
        let value = unique.borrow_mut();

        assert!("The value must not be already referred.", value.referred == false);
        value.referred = true;

        unique
    }

    pub fn borrow_box<'r>(&'r self) -> &'r T {
        unsafe { &*self._ptr }
    }

    pub fn borrow<'r>(&'r self) -> &'r T {
        &(self.borrow_box().value)
    }

    pub fn borrow_box_mut<'r>(&'r mut self) -> &'r mut T {
        unsafe { &mut *self._ptr }
    }

    pub fn borrow_mut<'r>(&'r mut self) -> &'r mut T {
        &mut (self.borrow_mut_box().value)
    }
}

impl Drop for CapabilityPoolCapability {
    fn drop(&mut self) {
        self.borrow_box_mut().referred = false;
    }
}
