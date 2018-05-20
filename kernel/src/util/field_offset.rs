use core::marker::PhantomData;
use core::mem;
use core::ops::Add;
use core::fmt;

/// Represents a pointer to a field of type `U` within the type `T`
pub struct FieldOffset<T, U>(
    /// Offset in bytes of the field within the struct
    usize,
    /// A pointer-to-member can be thought of as a function from
    /// `&T` to `&U` with matching lifetimes
    PhantomData<for<'a> Fn(&'a T) -> &'a U>
);

#[allow(dead_code)]
impl<T, U> FieldOffset<T, U> {
    /// Construct a field offset via a lambda which returns a reference
    /// to the field in question.
    ///
    /// The lambda *must not* access the value passed in.
    pub unsafe fn new<F: for<'a> FnOnce(&'a T) -> &'a U>(f: F) -> Self {
        // Construct a "fake" T. It's not valid, but the lambda shouldn't
        // actually access it (which is why this is unsafe)
        let x = mem::zeroed();
        let offset = {
            let x = &x;
            // Pass a reference to the zeroed T to the lambda
            // The lambda gives us back a reference to (what we hope is)
            // a field of T, of type U
            let y = f(x);
            // Compute the offset of the field via the difference between the
            // references `x` and `y`. Overflow is an error: in debug builds it
            // will be caught here, in release it will wrap around and be caught
            // on the next line.
            (y as *const U as usize) - (x as *const T as usize)
        };
        // Don't run destructor on "fake" T
        mem::forget(x);
        // Sanity check: ensure that the field offset plus the field size
        // is no greater than the size of the containing struct. This is
        // not sufficient to make the function *safe*, but it does catch
        // obvious errors like returning a reference to a boxed value,
        // which is owned by `T` and so has the correct lifetime, but is not
        // actually a field.
        assert!(offset + mem::size_of::<U>() <= mem::size_of::<T>());
        // Construct an instance using the offset
        Self::new_from_offset(offset)
    }
    /// Construct a field offset directly from a byte offset.
    pub unsafe fn new_from_offset(offset: usize) -> Self {
        FieldOffset(offset, PhantomData)
    }
    // Methods for applying the pointer to member
    /// Apply the field offset to a native pointer.
    pub fn apply_ptr<'a>(&self, x: *const T) -> *const U {
        ((x as usize) + self.0) as *const U
    }
    /// Apply the field offset to a native mutable pointer.
    pub fn apply_ptr_mut<'a>(&self, x: *mut T) -> *mut U {
        ((x as usize) + self.0) as *mut U
    }
    /// Apply the field offset to a reference.
    pub fn apply<'a>(&self, x: &'a T) -> &'a U {
        unsafe { &*self.apply_ptr(x) }
    }
    /// Apply the field offset to a mutable reference.
    pub fn apply_mut<'a>(&self, x: &'a mut T) -> &'a mut U {
        unsafe { &mut *self.apply_ptr_mut(x) }
    }
    /// Get the raw byte offset for this field offset.
    pub fn get_byte_offset(&self) -> usize {
        self.0
    }
    // Methods for unapplying the pointer to member
    /// Unapply the field offset to a native pointer.
    ///
    /// *Warning: very unsafe!*
    pub unsafe fn unapply_ptr<'a>(&self, x: *const U) -> *const T {
        ((x as usize) - self.0) as *const T
    }
    /// Unapply the field offset to a native mutable pointer.
    ///
    /// *Warning: very unsafe!*
    pub unsafe fn unapply_ptr_mut<'a>(&self, x: *mut U) -> *mut T {
        ((x as usize) - self.0) as *mut T
    }
    /// Unapply the field offset to a reference.
    ///
    /// *Warning: very unsafe!*
    pub unsafe fn unapply<'a>(&self, x: &'a U) -> &'a T {
        &*self.unapply_ptr(x)
    }
    /// Unapply the field offset to a mutable reference.
    ///
    /// *Warning: very unsafe!*
    pub unsafe fn unapply_mut<'a>(&self, x: &'a mut U) -> &'a mut T {
        &mut *self.unapply_ptr_mut(x)
    }
}

/// Allow chaining pointer-to-members.
///
/// Applying the resulting field offset is equivalent to applying the first
/// field offset, then applying the second field offset.
///
/// The requirements on the generic type parameters ensure this is a safe operation.
impl<T, U, V> Add<FieldOffset<U, V>> for FieldOffset<T, U> {
    type Output = FieldOffset<T, V>;

    fn add(self, other: FieldOffset<U, V>) -> FieldOffset<T, V> {
        FieldOffset(self.0 + other.0, PhantomData)
    }
}

/// The debug implementation prints the byte offset of the field in hexadecimal.
impl<T, U> fmt::Debug for FieldOffset<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "FieldOffset({:#x})", self.0)
    }
}

impl<T, U> Copy for FieldOffset<T, U> { }
impl<T, U> Clone for FieldOffset<T, U> {
    fn clone(&self) -> Self { *self }
}

/// This macro allows safe construction of a FieldOffset,
/// by generating a known to be valid lambda to pass to the
/// constructor. It takes a type and the identifier of a field
/// within that type as input.
///
/// Examples:
///
/// Offset of field `Foo().bar`
///
/// `offset_of!(Foo => bar)`
///
/// Offset of nested field `Foo().bar.x`
///
/// `offset_of!(Foo => bar: Bar => x)`
#[macro_export]
macro_rules! offset_of {
    ($t: path => $f: ident) => {
        unsafe { ::util::field_offset::FieldOffset::<$t, _>::new(|x| {
            let $t { ref $f, .. } = *x;
            $f
        }) }
    };
    ($t: path => $f: ident: $($rest: tt)*) => {
        offset_of!($t => $f) + offset_of!($($rest)*)
    };
}

#[cfg(test)]
mod tests {
    // Example structs
    #[derive(Debug)]
    struct Foo {
        a: u32,
        b: f64,
        c: bool
    }

    #[derive(Debug)]
    struct Bar {
        x: u32,
        y: Foo,
    }

    #[test]
    fn test_simple() {
        // Get a pointer to `b` within `Foo`
        let foo_b = offset_of!(Foo => b);

        // Construct an example `Foo`
        let mut x = Foo {
            a: 1,
            b: 2.0,
            c: false
        };

        // Apply the pointer to get at `b` and read it
        {
            let y = foo_b.apply(&x);
            assert!(*y == 2.0);
        }

        // Apply the pointer to get at `b` and mutate it
        {
            let y = foo_b.apply_mut(&mut x);
            *y = 42.0;
        }
        assert!(x.b == 42.0);
    }

    #[test]
    fn test_nested() {
        // Construct an example `Foo`
        let mut x = Bar {
            x: 0,
            y: Foo {
                a: 1,
                b: 2.0,
                c: false
            }
        };

        // Combine the pointer-to-members
        let bar_y_b = offset_of!(Bar => y: Foo => b);

        // Apply the pointer to get at `b` and mutate it
        {
            let y = bar_y_b.apply_mut(&mut x);
            *y = 42.0;
        }
        assert!(x.y.b == 42.0);
    }
}
