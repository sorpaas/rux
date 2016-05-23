use common::*;
use core::mem;

use super::{MemoryBlock, UntypedCapability,
            FrameCapability, GuardedCapability,
            PageTableCapability,
            Capability};

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

pub struct CapabilityPool([Option<CapabilityUnion>; CAPABILITY_POOL_COUNT]);

pub enum CapabilityUnion {
    /// Memory resources capabilities, all has its start and end address, and a
    /// next pointer to the next region (if available).
    ///
    /// A memory resources capability is essentially a pointer to a memory
    /// location.

    Untyped(UntypedCapability),
    Frame(FrameCapability),
    Guarded(GuardedCapability),
    PageTable(PageTableCapability),
}

pub trait CapabilityMove<T: Capability> {
    fn put(&mut self, T);
    fn take_one(&mut self) -> Option<T>;
    fn select<F>(&mut self, f: F) -> Option<T> where F: Fn(&T) -> bool;
    fn collect<F>(&mut self, mut f: F) where F: FnMut(T) -> Option<T>;
}

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

macro_rules! impl_move(
    ($cap_type:ty, $union_type:path) => (
        impl CapabilityMove<$cap_type> for CapabilityPool {
            fn put(&mut self, cap: $cap_type) {
                assert!(self.available_count() > 0);

                for i in 0..CAPABILITY_POOL_COUNT {
                    if self.0[i].is_none() {
                        self.0[i] = Some($union_type(cap));
                        return;
                    }
                }
            }

            fn take_one(&mut self) -> Option<$cap_type> {
                self.select(|x| true)
            }

            fn select<F>(&mut self, f: F) -> Option<$cap_type> where F: Fn(&$cap_type) -> bool {
                for i in 0..CAPABILITY_POOL_COUNT {
                    match self.0[i] {
                        Some($union_type(..)) => {
                            let union = unsafe { mem::replace(&mut self.0[i], None) };
                            match union.expect("") {
                                $union_type(x) => {
                                    if f(&x) == true {
                                        return Some(x)
                                    } else {
                                        unsafe { mem::replace(&mut self.0[i], Some($union_type(x)))};
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

            fn collect<F>(&mut self, mut f: F) where F: FnMut($cap_type) -> Option<$cap_type> {
                for i in 0..CAPABILITY_POOL_COUNT {
                    match self.0[i] {
                        Some($union_type(..)) => {
                            let union = unsafe { mem::replace(&mut self.0[i], None) };
                            match union.expect("") {
                                $union_type(x) => {
                                    unsafe { mem::replace(&mut self.0[i], f(x).and_then(|x| Some($union_type(x)))) };
                                }
                                _ => { panic!() }
                            }
                        }
                        _ => { }
                    }
                }
            }
        }
    )
);

impl_move!(UntypedCapability, CapabilityUnion::Untyped);
impl_move!(FrameCapability, CapabilityUnion::Frame);
impl_move!(GuardedCapability, CapabilityUnion::Guarded);
