//! Provides the wrapper type `Volatile`, which wraps a reference to any copy-able type and allows
//! for volatile memory access to wrapped value. Volatile memory accesses are never optimized away
//! by the compiler, and are useful in many low-level systems programming and concurrent contexts.
//!
//! The wrapper types *do not* enforce any atomicity guarantees; to also get atomicity, consider
//! looking at the `Atomic` wrapper type found in `libcore` or `libstd`.
//!
//! These wrappers do not depend on the standard library and never panic.

#![no_std]
#![cfg_attr(feature = "unstable", feature(core_intrinsics))]
#![cfg_attr(feature = "unstable", feature(const_generics))]
#![cfg_attr(feature = "unstable", allow(incomplete_features))]

//#![warn(missing_docs)]

use access::{ReadOnly, ReadWrite, Readable, Writable, WriteOnly};
#[cfg(feature = "unstable")]
use core::intrinsics;
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

/// Methods for volatile slices
impl<T, R, A> Volatile<R, A>
where
    R: Deref<Target = [T]>,
{
    pub fn index<'a, I>(&'a self, index: I) -> Volatile<&'a I::Output, A>
    where
        I: SliceIndex<[T]>,
        T: 'a,
    {
        Volatile {
            reference: self.reference.index(index),
            access: self.access,
        }
    }

    pub fn index_mut<'a, I>(&'a mut self, index: I) -> Volatile<&mut I::Output, A>
    where
        I: SliceIndex<[T]>,
        R: DerefMut,
        T: 'a,
    {
        Volatile {
            reference: self.reference.index_mut(index),
            access: self.access,
        }
    }

    /// Copies all elements from `self` into `dst`, using a volatile memcpy.
    ///
    /// The length of `dst` must be the same as `self`.
    ///
    /// The method is only available with the `nightly` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Panics
    ///
    /// This function will panic if the two slices have different lengths.
    ///
    /// ## Examples
    ///
    /// Copying two elements from a volatile slice:
    ///
    /// ```
    /// use volatile::Volatile;
    ///
    /// let src = [1, 2];
    /// // the `Volatile` type does not work with arrays, so convert `src` to a slice
    /// let slice = &src[..];
    /// let volatile = Volatile::new(slice);
    /// let mut dst = [5, 0, 0];
    ///
    /// // Because the slices have to be the same length,
    /// // we slice the destination slice from three elements
    /// // to two. It will panic if we don't do this.
    /// volatile.copy_into_slice(&mut dst[1..]);
    ///
    /// assert_eq!(src, [1, 2]);
    /// assert_eq!(dst, [5, 1, 2]);
    /// ```
    #[cfg(feature = "unstable")]
    pub fn copy_into_slice(&self, dst: &mut [T])
    where
        T: Copy,
    {
        assert_eq!(
            self.reference.len(),
            dst.len(),
            "destination and source slices have different lengths"
        );
        unsafe {
            intrinsics::volatile_copy_nonoverlapping_memory(
                dst.as_mut_ptr(),
                self.reference.as_ptr(),
                self.reference.len(),
            );
        }
    }

    /// Copies all elements from `src` into `self`, using a volatile memcpy.
    ///
    /// The length of `src` must be the same as `self`.
    ///
    /// The method is only available with the `nightly` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Panics
    ///
    /// This function will panic if the two slices have different lengths.
    ///
    /// ## Examples
    ///
    /// Copying two elements from a slice into a volatile slice:
    ///
    /// ```
    /// use volatile::Volatile;
    ///
    /// let src = [1, 2, 3, 4];
    /// let mut dst = [0, 0];
    /// // the `Volatile` type does not work with arrays, so convert `dst` to a slice
    /// let slice = &mut dst[..];
    /// let mut volatile = Volatile::new(slice);
    ///
    /// // Because the slices have to be the same length,
    /// // we slice the source slice from four elements
    /// // to two. It will panic if we don't do this.
    /// volatile.copy_from_slice(&src[2..]);
    ///
    /// assert_eq!(src, [1, 2, 3, 4]);
    /// assert_eq!(dst, [3, 4]);
    /// ```
    #[cfg(feature = "unstable")]
    pub fn copy_from_slice(&mut self, src: &[T])
    where
        T: Copy,
        R: DerefMut,
    {
        assert_eq!(
            self.reference.len(),
            src.len(),
            "destination and source slices have different lengths"
        );
        unsafe {
            intrinsics::volatile_copy_nonoverlapping_memory(
                self.reference.as_mut_ptr(),
                src.as_ptr(),
                self.reference.len(),
            );
        }
    }
}

/// Methods for converting arrays to slices
///
/// These methods are only available with the `nightly` feature enabled (requires a nightly
/// Rust compiler).
#[cfg(feature = "unstable")]
impl<R, A, T, const N: usize> Volatile<R, A>
where
    R: Deref<Target = [T; N]>,
{
    /// Converts an array reference to a shared slice.
    ///
    /// This makes it possible to use the methods defined on slices.
    ///
    /// ## Example
    ///
    /// Copying two elements from a volatile array reference using `copy_into_slice`:
    ///
    /// ```
    /// use volatile::Volatile;
    ///
    /// let src = [1, 2];
    /// let volatile = Volatile::new(&src);
    /// let mut dst = [0, 0];
    ///
    /// // convert the `Volatile<&[i32; 2]>` array reference to a `Volatile<&[i32]>` slice
    /// let volatile_slice = volatile.as_slice();
    /// // we can now use the slice methods
    /// volatile_slice.copy_into_slice(&mut dst);
    ///
    /// assert_eq!(dst, [1, 2]);
    /// ```
    pub fn as_slice(&self) -> Volatile<&[T], A> {
        Volatile {
            reference: &*self.reference,
            access: self.access,
        }
    }

    /// Converts a mutable array reference to a mutable slice.
    ///
    /// This makes it possible to use the methods defined on slices.
    ///
    /// ## Example
    ///
    /// Copying two elements from a slice into a mutable array reference:
    ///
    /// ```
    /// use volatile::Volatile;
    ///
    /// let src = [1, 2, 3, 4];
    /// let mut dst = [0, 0];
    /// let mut volatile = Volatile::new(&mut dst);
    ///
    /// // convert the `Volatile<&mut [i32; 2]>` array reference to a `Volatile<&mut [i32]>` slice
    /// let mut volatile_slice = volatile.as_mut_slice();
    /// // we can now use the slice methods
    /// volatile_slice.copy_from_slice(&src[2..]);
    ///
    /// assert_eq!(dst, [3, 4]);
    /// ```
    pub fn as_mut_slice(&mut self) -> Volatile<&mut [T], A>
    where
        R: DerefMut,
    {
        Volatile {
            reference: &mut *self.reference,
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
