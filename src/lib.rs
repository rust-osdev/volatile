#![cfg_attr(feature = "const_fn", feature(const_fn))]

//! Provides the wrapper type `Volatile`, which wraps a reference to any copy-able type and allows
//! for volatile memory access to wrapped value. Volatile memory accesses are never optimized away
//! by the compiler, and are useful in many low-level systems programming and concurrent contexts.
//!
//! The wrapper types *do not* enforce any atomicity guarantees; to also get atomicity, consider
//! looking at the `Atomic` wrapper type found in `libcore` or `libstd`.
//!
//! These wrappers do not depend on the standard library and never panic.

#![no_std]

pub use crate::access::{ReadWrite, Readable, Writable};
use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr,
    slice::SliceIndex,
};

mod access;

/// A wrapper type around a reference to a volatile variable.
///
/// Allows volatile reads and writes on the referenced value. The referenced value needs to
/// be `Copy`, as volatile reads and writes take and return copies of the value.
///
/// The size of this struct is the same as the size of the contained type.
///
/// TODO: read/write permissions
#[derive(Debug, Default, Clone)]
#[repr(transparent)]
pub struct Volatile<T, A = ReadWrite> {
    value: T,
    access: PhantomData<A>,
}

impl<T> Volatile<T> {
    /// Construct a new volatile instance wrapping the given value reference.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let value = 0u32;
    ///
    /// let volatile = Volatile::new(&value);
    /// assert_eq!(volatile.read(), 0);
    /// ```
    pub const fn new(value: T) -> Volatile<T> {
        Volatile {
            value,
            access: PhantomData,
        }
    }
}

impl<T: Copy, A> Volatile<&T, A> {
    /// Performs a volatile read of the contained value.
    ///
    /// Returns a copy of the read value. Volatile reads are guaranteed not to be optimized
    /// away by the compiler, but by themselves do not have atomic ordering
    /// guarantees. To also get atomicity, consider looking at the `Atomic` wrapper type.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let value = 42;
    /// let volatile = Volatile::new(&value);
    ///
    /// assert_eq!(volatile.read(), 42);
    /// ```
    pub fn read(&self) -> T
    where
        A: Readable,
    {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::read_volatile(self.value) }
    }
}

impl<T: Copy, A> Volatile<&mut T, A> {
    /// Performs a volatile read of the contained value.
    ///
    /// Returns a copy of the read value. Volatile reads are guaranteed not to be optimized
    /// away by the compiler, but by themselves do not have atomic ordering
    /// guarantees. To also get atomicity, consider looking at the `Atomic` wrapper type.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let mut value = 42;
    /// let volatile = Volatile::new(&mut value);
    ///
    /// assert_eq!(volatile.read(), 42);
    /// ```
    pub fn read(&self) -> T
    where
        A: Readable,
    {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::read_volatile(self.value) }
    }

    /// Performs a volatile write, setting the contained value to the given `value`.
    ///
    /// Volatile writes are guaranteed to not be optimized away by the compiler, but by
    /// themselves do not have atomic ordering guarantees. To also get atomicity, consider
    /// looking at the `Atomic` wrapper type.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let mut value = 42;
    /// let mut volatile = Volatile::new(&mut value);
    /// volatile.write(50);
    ///
    /// assert_eq!(volatile.read(), 50);
    /// ```
    pub fn write(&mut self, value: T)
    where
        A: Writable,
    {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::write_volatile(self.value, value) };
    }

    /// Updates the contained value using the given closure and volatile instructions.
    ///
    /// Performs a volatile read of the contained value, passes a mutable reference to it to the
    /// function `f`, and then performs a volatile write of the (potentially updated) value back to
    /// the contained value.
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let mut value = 42;
    /// let mut volatile = Volatile::new(&mut value);
    /// volatile.update(|val| *val += 1);
    ///
    /// assert_eq!(volatile.read(), 43);
    /// ```
    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
        A: Readable + Writable,
    {
        let mut value = self.read();
        f(&mut value);
        self.write(value);
    }
}

impl<T: Copy, A> Volatile<&[T], A> {
    pub fn index<I>(&self, index: I) -> Volatile<&I::Output, A>
    where
        I: SliceIndex<[T]>,
    {
        Volatile {
            value: self.value.index(index),
            access: self.access,
        }
    }
}

impl<T: Copy, A> Volatile<&mut [T], A> {
    pub fn index<I>(&self, index: I) -> Volatile<&I::Output, A>
    where
        I: SliceIndex<[T]>,
    {
        Volatile {
            value: self.value.index(index),
            access: self.access,
        }
    }

    pub fn index_mut<I>(&mut self, index: I) -> Volatile<&mut I::Output, A>
    where
        I: SliceIndex<[T]>,
    {
        Volatile {
            value: self.value.index_mut(index),
            access: self.access,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Volatile;

    #[test]
    fn test_read() {
        let val = 42;
        assert_eq!(Volatile::new(&val).read(), 42);
    }

    #[test]
    fn test_write() {
        let mut val = 50;
        let mut volatile = Volatile::new(&mut val);
        volatile.write(50);
        assert_eq!(val, 50);
    }

    #[test]
    fn test_update() {
        let mut val = 42;
        let mut volatile = Volatile::new(&mut val);
        volatile.update(|v| *v += 1);
        assert_eq!(val, 43);
    }

    #[test]
    fn test_slice() {
        let mut val = [1, 2, 3];
        let mut volatile = Volatile::new(&mut val[..]);
        volatile.index_mut(0).update(|v| *v += 1);
        assert_eq!(val, [2, 2, 3]);
    }
}
