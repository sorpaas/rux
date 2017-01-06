use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

macro_rules! addr_common {
    ( $t:ty, $e:expr ) => {
        impl Add<usize> for $t {
            type Output = Self;

            fn add(self, _rhs: usize) -> Self {
                Self::from(self.into(): usize + _rhs)
            }
        }

        impl AddAssign<usize> for $t {
            fn add_assign(&mut self, _rhs: usize) {
                self.0 = self.0 + (_rhs as u64);
            }
        }

        impl fmt::Binary for $t {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl fmt::LowerHex for $t {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl fmt::Octal for $t {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl fmt::UpperHex for $t {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<usize> for $t {
            fn from(v: usize) -> Self { $e(v as u64) }
        }

        impl From<u64> for $t {
            fn from(v: u64) -> Self { $e(v as u64) }
        }

        impl From<u32> for $t {
            fn from(v: u32) -> Self { $e(v as u64) }
        }

        impl Into<usize> for $t {
            fn into(self) -> usize { self.0 as usize }
        }

        impl Into<u64> for $t {
            fn into(self) -> u64 { self.0 as u64 }
        }

        impl Into<u32> for $t {
            fn into(self) -> u32 { self.0 as u32 }
        }

        impl $t {
            pub const fn new(v: u64) -> $t { $e(v) }
        }
    }
}

/// Represent a physical memory address.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PAddr(u64);

addr_common!(PAddr, PAddr);

/// Represent a virtual (linear) memory address.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VAddr(u64);

addr_common!(VAddr, VAddr);
