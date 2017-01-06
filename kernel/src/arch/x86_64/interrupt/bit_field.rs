#![feature(const_fn)]
#![no_std]

use core::mem::size_of;

/// Represent a bit field.
#[derive(Debug, Clone, Copy)]
pub struct BitField<T: Number>(T);

impl<T> BitField<T>
    where T: Number
{
    /// Create a bit field using value.
    pub const fn new(value: T) -> BitField<T> {
        BitField(value)
    }

    /// Return the underlying number of the bit field.
    pub fn bits(&self) -> T {
        self.0
    }

    /// Get a specific bit in the bit field.
    pub fn get_bit(&self, bit: u8) -> bool {
        assert!(bit < self.length());
        self.get_range(bit..(bit + 1)) == T::one()
    }

    /// Get a range of bits.
    pub fn get_range(&self, range: Range<u8>) -> T {
        assert!(range.start < self.length());
        assert!(range.end <= self.length());
        assert!(range.start < range.end);

        // shift away high bits
        let bits = self.0 << (self.length() - range.end) >> (self.length() - range.end);

        // shift away low bits
        bits >> range.start
    }

    /// Set a single bit.
    pub fn set_bit(&mut self, bit: u8, value: bool) -> &mut Self {
        assert!(bit < self.length());
        if value {
            self.0 |= T::one() << bit;
        } else {
            self.0 &= !(T::one() << bit);
        }
        self
    }

    /// Set a range of bits.
    pub fn set_range(&mut self, range: Range<u8>, value: T) -> &mut Self {
        assert!(range.start < self.length());
        assert!(range.end <= self.length());
        assert!(range.start < range.end);
        assert!(value << (self.length() - (range.end - range.start)) >>
                (self.length() - (range.end - range.start)) == value,
                "value too big");

        let bitmask: T = !(!T::zero() << (self.length() - range.end) >>
                           (self.length() - range.end) >>
                           range.start << range.start);

        let bits = self.0 & bitmask;
        // set bits
        self.0 = bits | (value << range.start);

        self
    }

    /// Get the length of the bit field.
    fn length(&self) -> u8 {
        size_of::<T>() as u8 * 8
    }
}

use core::ops::{Range, Shl, Shr, BitAnd, BitOr, BitOrAssign, BitAndAssign, Not};
use core::fmt::Debug;

/// Represent a number trait.
pub trait Number: Debug + Copy + Eq +
    Not<Output=Self> + Shl<u8, Output=Self> + Shr<u8, Output=Self> +
    BitAnd<Self, Output=Self> + BitOr<Self, Output=Self>  + BitAndAssign + BitOrAssign {

    fn zero() -> Self;
    fn one() -> Self;
}

macro_rules! number_impl {
    ($($t:ty)*) => ($(
        impl Number for $t {
            fn zero() -> Self { 0 }
            fn one() -> Self { 1 }
        }
    )*)
}

number_impl! { u8 u16 u32 u64 usize }
