use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PAddr(u64);

addr_common!(PAddr);

impl PAddr {
    /// Convert to `u64`
    pub const fn as_u64(&self) -> u64 {
        self.0 as u64
    }

    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
    
    /// Convert from `u64`
    pub const fn from_u64(v: u64) -> Self {
        PAddr(v as u64)
    }

    pub const fn from_u32(v: u32) -> Self {
        PAddr(v as u64)
    }

    pub const fn from_usize(v: usize) -> Self {
        PAddr(v as u64)
    }
}

/// Represent a virtual (linear) memory address
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VAddr(u64);

addr_common!(VAddr);

impl VAddr {
    /// Convert to `u64`
    pub const fn as_u64(&self) -> u64 {
        self.0 as u64
    }

    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
    
    /// Convert from `u64`
    pub const fn from_u64(v: u64) -> Self {
        VAddr(v as u64)
    }

    pub const fn from_u32(v: u32) -> Self {
        VAddr(v as u64)
    }

    pub const fn from_usize(v: usize) -> Self {
        VAddr(v as u64)
    }
}
