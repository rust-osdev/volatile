//! Provides the wrapper type `Volatile`, which wraps a reference to any copy-able type and allows
//! for volatile memory access to wrapped value. Volatile memory accesses are never optimized away
//! by the compiler, and are useful in many low-level systems programming and concurrent contexts.
//!
//! The wrapper types *do not* enforce any atomicity guarantees; to also get atomicity, consider
//! looking at the `Atomic` wrapper type found in `libcore` or `libstd`.
//!
//! These wrappers do not depend on the standard library and never panic.

#![no_std]

use access::{ReadOnly, ReadWrite, Readable, Writable, WriteOnly};
use core::{
    marker::PhantomData,
    ops::Deref,
    ops::{DerefMut, Index, IndexMut},
    ptr,
    slice::SliceIndex,
};

/// Allows creating read-only and write-only `Volatile` values.
pub mod access;

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
pub struct Volatile<R, A = ReadWrite> {
    reference: R,
    access: PhantomData<A>,
}

/// Construction functions
impl<R> Volatile<R> {
    /// Construct a new volatile instance wrapping the given reference.
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
    pub const fn new(reference: R) -> Volatile<R> {
        Volatile {
            reference,
            access: PhantomData,
        }
    }

    pub const fn new_read_only(reference: R) -> Volatile<R, ReadOnly> {
        Volatile {
            reference,
            access: PhantomData,
        }
    }

    pub const fn new_write_only(reference: R) -> Volatile<R, WriteOnly> {
        Volatile {
            reference,
            access: PhantomData,
        }
    }
}

/// Methods for references to `Copy` types
impl<R, T, A> Volatile<R, A>
where
    R: Deref<Target = T>,
    T: Copy,
{
    /// Performs a volatile read of the contained value.
    ///
    /// Returns a copy of the read value. Volatile reads are guaranteed not to be optimized
    /// away by the compiler, but by themselves do not have atomic ordering
    /// guarantees. To also get atomicity, consider looking at the `Atomic` wrapper type.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let value = 42;
    /// let shared_reference = Volatile::new(&value);
    /// assert_eq!(shared_reference.read(), 42);
    ///
    /// let mut value = 50;
    /// let mut_reference = Volatile::new(&mut value);
    /// assert_eq!(mut_reference.read(), 50);
    /// ```
    pub fn read(&self) -> T
    where
        A: Readable,
    {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::read_volatile(&*self.reference) }
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
        R: DerefMut,
    {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::write_volatile(&mut *self.reference, value) };
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
        A: Readable + Writable,
        R: DerefMut,
        F: FnOnce(&mut T),
    {
        let mut value = self.read();
        f(&mut value);
        self.write(value);
    }
}

impl<T, A> Volatile<&[T], A> {
    pub fn index<I>(&self, index: I) -> Volatile<&I::Output, A>
    where
        I: SliceIndex<[T]>,
    {
        Volatile {
            reference: self.reference.index(index),
            access: self.access,
        }
    }
}

impl<T, A> Volatile<&mut [T], A> {
    pub fn index<I>(&self, index: I) -> Volatile<&I::Output, A>
    where
        I: SliceIndex<[T]>,
    {
        Volatile {
            reference: self.reference.index(index),
            access: self.access,
        }
    }

    pub fn index_mut<I>(&mut self, index: I) -> Volatile<&mut I::Output, A>
    where
        I: SliceIndex<[T]>,
    {
        Volatile {
            reference: self.reference.index_mut(index),
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
