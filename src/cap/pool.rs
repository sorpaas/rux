use common::*;
use core::mem;

use super::Capability;
use super::{CapabilityPool, CapabilityUnion, CapabilityMove};
use super::{UntypedCapability, KernelReservedBlockCapability};

macro_rules! init_array(
    ($ty:ty, $len:expr, $val:expr) => (
        {
            let mut array: [$ty; $len] = unsafe { ::core::mem::uninitialized() };
            for i in array.iter_mut() {
                unsafe { ::core::ptr::write(i, $val); }
            }
            array
        }
    )
);

impl CapabilityPool {
    pub fn new() -> CapabilityPool {
        CapabilityPool(init_array!(Option<CapabilityUnion>, CAPABILITY_POOL_COUNT, None))
    }

    pub fn reset(&mut self) {
        for i in 0..CAPABILITY_POOL_COUNT {
            self.0[i] = None;
        }
    }

    pub fn available_count(&self) -> usize {
        let mut count = 0;

        for i in 0..CAPABILITY_POOL_COUNT {
            if self.0[i].is_none() {
                count = count + 1;
            }
        }

        count
    }
}

impl CapabilityMove<UntypedCapability> for CapabilityPool {
    fn put(&mut self, cap: UntypedCapability) {
        assert!(self.available_count() > 0);

        for i in 0..CAPABILITY_POOL_COUNT {
            if self.0[i].is_none() {
                self.0[i] = Some(CapabilityUnion::Untyped(cap));
                return;
            }
        }
    }

    fn take_one(&mut self) -> Option<UntypedCapability> {
        self.select(|x| true)
    }

    fn select<F>(&mut self, f: F) -> Option<UntypedCapability> where F: Fn(&UntypedCapability) -> bool {
        for i in 0..CAPABILITY_POOL_COUNT {
            match self.0[i] {
                Some(CapabilityUnion::Untyped(..)) => {
                    let union = unsafe { mem::replace(&mut self.0[i], None) };
                    match union.expect("") {
                        CapabilityUnion::Untyped(x) => {
                            if f(&x) == true {
                                return Some(x)
                            } else {
                                unsafe { mem::replace(&mut self.0[i], Some(CapabilityUnion::Untyped(x)))};
                            }
                        }
                        _ => { panic!() }
                    }
                }
                _ => { }
            }
        }
        None
    }
}

impl CapabilityMove<KernelReservedBlockCapability> for CapabilityPool {
    fn put(&mut self, cap: KernelReservedBlockCapability) {
        assert!(self.available_count() > 0);

        for i in 0..CAPABILITY_POOL_COUNT {
            if self.0[i].is_none() {
                self.0[i] = Some(CapabilityUnion::KernelReserved(cap));
                return;
            }
        }
    }

    fn take_one(&mut self) -> Option<KernelReservedBlockCapability> {
        self.select(|x| true)
    }

    fn select<F>(&mut self, f: F) -> Option<KernelReservedBlockCapability> where F: Fn(&KernelReservedBlockCapability) -> bool {
        for i in 0..CAPABILITY_POOL_COUNT {
            match self.0[i] {
                Some(CapabilityUnion::KernelReserved(..)) => {
                    let union = unsafe { mem::replace(&mut self.0[i], None) };
                    match union.expect("") {
                        CapabilityUnion::KernelReserved(x) => {
                            if f(&x) == true {
                                return Some(x)
                            } else {
                                unsafe { mem::replace(&mut self.0[i], Some(CapabilityUnion::KernelReserved(x)))};
                            }
                        }
                        _ => { panic!() }
                    }
                }
                _ => { }
            }
        }
        None
    }
}
