//! Provides the wrapper type `Volatile`, which wraps a reference to any copy-able type and allows
//! for volatile memory access to wrapped value. Volatile memory accesses are never optimized away
//! by the compiler, and are useful in many low-level systems programming and concurrent contexts.
//!
//! The wrapper types *do not* enforce any atomicity guarantees; to also get atomicity, consider
//! looking at the `Atomic` wrapper types found in `libcore` or `libstd`.

#![no_std]
#![cfg_attr(feature = "unstable", feature(core_intrinsics))]
#![cfg_attr(feature = "unstable", feature(const_generics))]
#![cfg_attr(feature = "unstable", feature(slice_range))]
#![cfg_attr(feature = "unstable", feature(slice_ptr_get))]
#![cfg_attr(feature = "unstable", feature(slice_ptr_len))]
#![cfg_attr(feature = "unstable", allow(incomplete_features))]
#![cfg_attr(all(feature = "unstable", test), feature(slice_as_chunks))]
#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

use access::{ReadOnly, ReadWrite, Readable, Writable, WriteOnly};
use core::{fmt, marker::PhantomData, ptr};
#[cfg(feature = "unstable")]
use core::{
    intrinsics,
    ops::{Range, RangeBounds},
    slice::{range, SliceIndex},
};

/// Allows creating read-only and write-only `Volatile` values.
pub mod access;

/// Wraps a reference to make accesses to the referenced value volatile.
///
/// Allows volatile reads and writes on the referenced value. The referenced value needs to
/// be `Copy` for reading and writing, as volatile reads and writes take and return copies
/// of the value.
///
/// Since not all volatile resources (e.g. memory mapped device registers) are both readable
/// and writable, this type supports limiting the allowed access types through an optional second
/// generic parameter `A` that can be one of `ReadWrite`, `ReadOnly`, or `WriteOnly`. It defaults
/// to `ReadWrite`, which allows all operations.
///
/// The size of this struct is the same as the size of the contained reference.
#[repr(transparent)]
pub struct Volatile<T, A = ReadWrite>
where
    T: ?Sized,
{
    pointer: *mut T,
    access: PhantomData<A>,
}

/// Constructor functions for creating new values
///
/// These functions allow to construct a new `Volatile` instance from a reference type. While
/// the `new` function creates a `Volatile` instance with unrestricted access, there are also
/// functions for creating read-only or write-only instances.
impl<T> Volatile<T>
where
    T: ?Sized,
{
    /// Constructs a new volatile instance wrapping the given reference.
    ///
    /// While it is possible to construct `Volatile` instances from arbitrary values (including
    /// non-reference values), most of the methods are only available when the wrapped type is
    /// a reference. The only reason that we don't forbid non-reference types in the constructor
    /// functions is that the Rust compiler does not support trait bounds on generic `const`
    /// functions yet. When this becomes possible, we will release a new version of this library
    /// with removed support for non-references. For these reasons it is recommended to use
    /// the `Volatile` type only with references.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let mut value = 0u32;
    ///
    /// let mut volatile = Volatile::new(&mut value);
    /// volatile.write(1);
    /// assert_eq!(volatile.read(), 1);
    /// ```
    pub unsafe fn new<A>(pointer: *mut T, access: A) -> Volatile<T, A> {
        let _: A = access;
        Volatile {
            pointer,
            access: PhantomData,
        }
    }

    pub unsafe fn from_ptr(pointer: *const T) -> Volatile<T, ReadOnly> {
        unsafe { Volatile::new(pointer as *mut _, ReadOnly) }
    }

    pub unsafe fn from_mut_ptr(pointer: *mut T) -> Volatile<T> {
        unsafe { Volatile::new(pointer, ReadWrite) }
    }
}

/// Methods for references to `Copy` types
impl<'a, T, A> Volatile<T, A>
where
    T: Copy + ?Sized,
{
    /// Performs a volatile read of the contained value.
    ///
    /// Returns a copy of the read value. Volatile reads are guaranteed not to be optimized
    /// away by the compiler, but by themselves do not have atomic ordering
    /// guarantees. To also get atomicity, consider looking at the `Atomic` wrapper types of
    /// the standard/`core` library.
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
        // UNSAFE: Safe, as ... TODO
        unsafe { ptr::read_volatile(self.pointer) }
    }

    /// Performs a volatile write, setting the contained value to the given `value`.
    ///
    /// Volatile writes are guaranteed to not be optimized away by the compiler, but by
    /// themselves do not have atomic ordering guarantees. To also get atomicity, consider
    /// looking at the `Atomic` wrapper types of the standard/`core` library.
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
        // UNSAFE: Safe, as ... TODO
        unsafe { ptr::write_volatile(self.pointer, value) };
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
        F: FnOnce(&mut T),
    {
        let mut value = self.read();
        f(&mut value);
        self.write(value);
    }
}

/// Method for extracting the wrapped value.
impl<'a, T, A> Volatile<T, A>
where
    T: ?Sized,
{
    /// Extracts the inner value stored in the wrapper type.
    ///
    /// This method gives direct access to the wrapped reference and thus allows
    /// non-volatile access again. This is seldom what you want since there is usually
    /// a reason that a reference is wrapped in `Volatile`. However, in some cases it might
    /// be required or useful to use the `read_volatile`/`write_volatile` pointer methods of
    /// the standard library directly, which this method makes possible.
    ///
    /// Since no memory safety violation can occur when accessing the referenced value using
    /// non-volatile operations, this method is safe. However, it _can_ lead to bugs at the
    /// application level, so this method should be used with care.
    ///
    /// ## Example
    ///
    /// ```
    /// use volatile::Volatile;
    ///
    /// let mut value = 42;
    /// let mut volatile = Volatile::new(&mut value);
    /// volatile.write(50);
    /// let unwrapped: &mut i32 = volatile.extract_inner();
    ///
    /// assert_eq!(*unwrapped, 50); // non volatile access, be careful!
    /// ```
    pub fn as_ptr(&self) -> *mut T {
        self.pointer
    }
}

/// Transformation methods for accessing struct fields
impl<T, A> Volatile<T, A>
where
    T: ?Sized,
{
    /// Constructs a new `Volatile` reference by mapping the wrapped value.
    ///
    /// This method is useful for accessing individual fields of volatile structs.
    ///
    /// Note that this method gives temporary access to the wrapped reference, which allows
    /// accessing the value in a non-volatile way. This is normally not what you want, so
    /// **this method should only be used for reference-to-reference transformations**.
    ///
    /// ## Examples
    ///
    /// Accessing a struct field:
    ///
    /// ```
    /// use volatile::Volatile;
    ///
    /// struct Example { field_1: u32, field_2: u8, }
    /// let mut value = Example { field_1: 15, field_2: 255 };
    /// let mut volatile = Volatile::new(&mut value);
    ///
    /// // construct a volatile reference to a field
    /// let field_2 = volatile.map(|example| &example.field_2);
    /// assert_eq!(field_2.read(), 255);
    /// ```
    ///
    /// Don't misuse this method to do a non-volatile read of the referenced value:
    ///
    /// ```
    /// use volatile::Volatile;
    ///
    /// let mut value = 5;
    /// let mut volatile = Volatile::new(&mut value);
    ///
    /// // DON'T DO THIS:
    /// let mut readout = 0;
    /// volatile.map(|value| {
    ///    readout = *value; // non-volatile read, might lead to bugs
    ///    value
    /// });
    /// ```
    pub unsafe fn map<F, U>(&self, f: F) -> Volatile<U, ReadOnly>
    where
        F: FnOnce(*mut T) -> *mut U,
        U: ?Sized,
    {
        Volatile {
            pointer: f(self.pointer),
            access: PhantomData,
        }
    }

    pub unsafe fn map_mut<F, U>(&mut self, f: F) -> Volatile<U, A>
    where
        F: FnOnce(*mut T) -> *mut U,
        U: ?Sized,
    {
        Volatile {
            pointer: f(self.pointer),
            access: self.access,
        }
    }
}

/// Methods for volatile slices
#[cfg(feature = "unstable")]
impl<'a, T, A> Volatile<[T], A> {
    /// Applies the index operation on the wrapped slice.
    ///
    /// Returns a shared `Volatile` reference to the resulting subslice.
    ///
    /// This is a convenience method for the `map(|slice| slice.index(index))` operation, so it
    /// has the same behavior as the indexing operation on slice (e.g. panic if index is
    /// out-of-bounds).
    ///
    /// ## Examples
    ///
    /// Accessing a single slice element:
    ///
    /// ```
    /// use volatile::Volatile;
    ///
    /// let array = [1, 2, 3];
    /// let slice = &array[..];
    /// let volatile = Volatile::new(slice);
    /// assert_eq!(volatile.index(1).read(), 2);
    /// ```
    ///
    /// Accessing a subslice:
    ///
    /// ```
    /// use volatile::Volatile;
    ///
    /// let array = [1, 2, 3];
    /// let slice = &array[..];
    /// let volatile = Volatile::new(slice);
    /// let subslice = volatile.index(1..);
    /// assert_eq!(subslice.index(0).read(), 2);
    /// ```
    pub fn index<I>(&self, index: I) -> Volatile<I::Output, ReadOnly>
    where
        I: SliceIndex<[T]>,
    {
        unsafe { self.map(|slice| slice.get_unchecked_mut(index)) }
    }

    pub fn index_mut<I>(&mut self, index: I) -> Volatile<I::Output, A>
    where
        I: SliceIndex<[T]>,
    {
        unsafe { self.map_mut(|slice| slice.get_unchecked_mut(index)) }
    }

    /// Copies all elements from `self` into `dst`, using a volatile memcpy.
    ///
    /// The length of `dst` must be the same as `self`.
    ///
    /// The method is only available with the `unstable` feature enabled (requires a nightly
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
    pub fn copy_into_slice(&self, dst: &mut [T])
    where
        T: Copy,
    {
        let len = self.pointer.len();
        assert_eq!(
            len,
            dst.len(),
            "destination and source slices have different lengths"
        );
        unsafe {
            intrinsics::volatile_copy_nonoverlapping_memory(
                dst.as_mut_ptr(),
                self.pointer.as_mut_ptr(),
                len,
            );
        }
    }

    /// Copies all elements from `src` into `self`, using a volatile memcpy.
    ///
    /// The length of `src` must be the same as `self`.
    ///
    /// This method is similar to the `slice::copy_from_slice` method of the standard library. The
    /// difference is that this method performs a volatile copy.
    ///
    /// The method is only available with the `unstable` feature enabled (requires a nightly
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
    pub fn copy_from_slice(&mut self, src: &[T])
    where
        T: Copy,
    {
        let len = self.pointer.len();
        assert_eq!(
            len,
            src.len(),
            "destination and source slices have different lengths"
        );
        unsafe {
            intrinsics::volatile_copy_nonoverlapping_memory(
                self.pointer.as_mut_ptr(),
                src.as_ptr(),
                len,
            );
        }
    }

    /// Copies elements from one part of the slice to another part of itself, using a
    /// volatile `memmove`.
    ///
    /// `src` is the range within `self` to copy from. `dest` is the starting index of the
    /// range within `self` to copy to, which will have the same length as `src`. The two ranges
    /// may overlap. The ends of the two ranges must be less than or equal to `self.len()`.
    ///
    /// This method is similar to the `slice::copy_within` method of the standard library. The
    /// difference is that this method performs a volatile copy.
    ///
    /// This method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Panics
    ///
    /// This function will panic if either range exceeds the end of the slice, or if the end
    /// of `src` is before the start.
    ///
    /// ## Examples
    ///
    /// Copying four bytes within a slice:
    ///
    /// ```
    /// use volatile::Volatile;
    ///
    /// let mut byte_array = *b"Hello, World!";
    /// let mut slice: &mut [u8] = &mut byte_array[..];
    /// let mut volatile = Volatile::from_mut_ptr(slice);
    ///
    /// volatile.copy_within(1..5, 8);
    ///
    /// assert_eq!(&byte_array, b"Hello, Wello!");
    pub fn copy_within(&mut self, src: impl RangeBounds<usize>, dest: usize)
    where
        T: Copy,
    {
        let len = self.pointer.len();
        // implementation taken from https://github.com/rust-lang/rust/blob/683d1bcd405727fcc9209f64845bd3b9104878b8/library/core/src/slice/mod.rs#L2726-L2738
        let Range {
            start: src_start,
            end: src_end,
        } = range(src, ..len);
        let count = src_end - src_start;
        assert!(dest <= len - count, "dest is out of bounds");
        // SAFETY: the conditions for `volatile_copy_memory` have all been checked above,
        // as have those for `ptr::add`.
        unsafe {
            intrinsics::volatile_copy_memory(
                self.pointer.as_mut_ptr().add(dest),
                self.pointer.as_mut_ptr().add(src_start),
                count,
            );
        }
    }

    pub fn split_at(&self, mid: usize) -> (Volatile<[T], ReadOnly>, Volatile<[T], ReadOnly>) {
        assert!(mid <= self.pointer.len());
        // SAFETY: `[ptr; mid]` and `[mid; len]` are inside `self`, which
        // fulfills the requirements of `from_raw_parts_mut`.
        unsafe { self.split_at_unchecked(mid) }
    }

    pub fn split_at_mut(&mut self, mid: usize) -> (Volatile<[T], A>, Volatile<[T], A>) {
        assert!(mid <= self.pointer.len());
        // SAFETY: `[ptr; mid]` and `[mid; len]` are inside `self`, which
        // fulfills the requirements of `from_raw_parts_mut`.
        unsafe { self.split_at_mut_unchecked(mid) }
    }

    unsafe fn split_at_unchecked(
        &self,
        mid: usize,
    ) -> (Volatile<[T], ReadOnly>, Volatile<[T], ReadOnly>) {
        // SAFETY: Caller has to check that `0 <= mid <= self.len()`
        unsafe {
            (
                Volatile {
                    pointer: { (self.pointer).get_unchecked_mut(..mid) },
                    access: PhantomData,
                },
                Volatile {
                    pointer: { (self.pointer).get_unchecked_mut(mid..) },
                    access: PhantomData,
                },
            )
        }
    }

    unsafe fn split_at_mut_unchecked(
        &mut self,
        mid: usize,
    ) -> (Volatile<[T], A>, Volatile<[T], A>) {
        let len = self.pointer.len();
        let ptr = self.pointer.as_mut_ptr();

        // SAFETY: Caller has to check that `0 <= mid <= self.len()`.
        //
        // `[ptr; mid]` and `[mid; len]` are not overlapping, so returning a mutable reference
        // is fine.
        unsafe {
            (
                Volatile {
                    pointer: { ptr::slice_from_raw_parts_mut(ptr, mid) },
                    access: self.access,
                },
                Volatile {
                    pointer: { ptr::slice_from_raw_parts_mut(ptr.add(mid), len - mid) },
                    access: self.access,
                },
            )
        }
    }

    pub fn as_chunks<const N: usize>(
        &self,
    ) -> (Volatile<[[T; N]], ReadOnly>, Volatile<[T], ReadOnly>) {
        assert_ne!(N, 0);
        let len = self.pointer.len() / N;
        let (multiple_of_n, remainder) = self.split_at(len * N);
        // SAFETY: We already panicked for zero, and ensured by construction
        // that the length of the subslice is a multiple of N.
        let array_slice = unsafe { multiple_of_n.as_chunks_unchecked_by_val() };
        (array_slice, remainder)
    }

    pub unsafe fn as_chunks_unchecked<const N: usize>(&self) -> Volatile<[[T; N]], ReadOnly> {
        debug_assert_ne!(N, 0);
        debug_assert_eq!(self.pointer.len() % N, 0);
        let new_len =
            // SAFETY: Our precondition is exactly what's needed to call this
            unsafe { crate::intrinsics::exact_div(self.pointer.len(), N) };
        // SAFETY: We cast a slice of `new_len * N` elements into
        // a slice of `new_len` many `N` elements chunks.
        let pointer = ptr::slice_from_raw_parts_mut(self.pointer.as_mut_ptr().cast(), new_len);
        Volatile {
            pointer: pointer,
            access: PhantomData,
        }
    }

    pub fn as_chunks_mut<const N: usize>(&mut self) -> (Volatile<[[T; N]], A>, Volatile<[T], A>) {
        assert_ne!(N, 0);
        let len = self.pointer.len() / N;
        let (multiple_of_n, remainder) = self.split_at_mut(len * N);
        // SAFETY: We already panicked for zero, and ensured by construction
        // that the length of the subslice is a multiple of N.
        let array_slice = unsafe { multiple_of_n.as_chunks_unchecked_by_val() };
        (array_slice, remainder)
    }

    pub unsafe fn as_chunks_unchecked_mut<const N: usize>(&mut self) -> Volatile<[[T; N]], A> {
        debug_assert_ne!(N, 0);
        debug_assert_eq!(self.pointer.len() % N, 0);
        let new_len =
            // SAFETY: Our precondition is exactly what's needed to call this
            unsafe { crate::intrinsics::exact_div(self.pointer.len(), N) };
        // SAFETY: We cast a slice of `new_len * N` elements into
        // a slice of `new_len` many `N` elements chunks.
        let pointer = ptr::slice_from_raw_parts_mut(self.pointer.as_mut_ptr().cast(), new_len);
        Volatile {
            pointer,
            access: self.access,
        }
    }

    pub unsafe fn as_chunks_unchecked_by_val<const N: usize>(self) -> Volatile<[[T; N]], A> {
        debug_assert_ne!(N, 0);
        debug_assert_eq!(self.pointer.len() % N, 0);
        let new_len =
            // SAFETY: Our precondition is exactly what's needed to call this
            unsafe { crate::intrinsics::exact_div(self.pointer.len(), N) };
        // SAFETY: We cast a slice of `new_len * N` elements into
        // a slice of `new_len` many `N` elements chunks.
        let pointer = ptr::slice_from_raw_parts_mut(self.pointer.as_mut_ptr().cast(), new_len);
        Volatile {
            pointer,
            access: self.access,
        }
    }
}

/// Methods for volatile byte slices
#[cfg(feature = "unstable")]
impl<A> Volatile<[u8], A> {
    /// Sets all elements of the byte slice to the given `value` using a volatile `memset`.
    ///
    /// This method is similar to the `slice::fill` method of the standard library, with the
    /// difference that this method performs a volatile write operation. Another difference
    /// is that this method is only available for byte slices (not general `&mut [T]` slices)
    /// because there currently isn't a instrinsic function that allows non-`u8` values.
    ///
    /// This method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Example
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let mut buf = Volatile::new(vec![0; 10]);
    /// buf.fill(1);
    /// assert_eq!(buf.extract_inner(), vec![1; 10]);
    /// ```
    pub fn fill(&mut self, value: u8) {
        unsafe {
            intrinsics::volatile_set_memory(self.pointer.as_mut_ptr(), value, self.pointer.len());
        }
    }
}

/// Methods for converting arrays to slices
///
/// These methods are only available with the `unstable` feature enabled (requires a nightly
/// Rust compiler).
#[cfg(feature = "unstable")]
impl<T, A, const N: usize> Volatile<[T; N], A> {
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
    pub fn as_slice(&self) -> Volatile<[T], ReadOnly> {
        unsafe { self.map(|array| ptr::slice_from_raw_parts_mut(array as *mut T, N)) }
    }
}

/// Methods for restricting access.
impl<'a, T> Volatile<T>
where
    T: ?Sized,
{
    /// Restricts access permissions to read-only.
    ///
    /// ## Example
    ///
    /// ```
    /// use volatile::Volatile;
    ///
    /// let mut value: i16 = -4;
    /// let mut volatile = Volatile::new(&mut value);
    ///
    /// let read_only = volatile.read_only();
    /// assert_eq!(read_only.read(), -4);
    /// // read_only.write(10); // compile-time error
    /// ```
    pub fn read_only(self) -> Volatile<T, ReadOnly> {
        Volatile {
            pointer: self.pointer,
            access: PhantomData,
        }
    }

    /// Restricts access permissions to write-only.
    ///
    /// ## Example
    ///
    /// Creating a write-only reference to a struct field:
    ///
    /// ```
    /// use volatile::Volatile;
    ///
    /// struct Example { field_1: u32, field_2: u8, }
    /// let mut value = Example { field_1: 15, field_2: 255 };
    /// let mut volatile = Volatile::new(&mut value);
    ///
    /// // construct a volatile write-only reference to `field_2`
    /// let mut field_2 = volatile.map_mut(|example| &mut example.field_2).write_only();
    /// field_2.write(14);
    /// // field_2.read(); // compile-time error
    /// ```
    pub fn write_only(self) -> Volatile<T, WriteOnly> {
        Volatile {
            pointer: self.pointer,
            access: PhantomData,
        }
    }
}

impl<T, A> fmt::Debug for Volatile<T, A>
where
    T: Copy + fmt::Debug + ?Sized,
    A: Readable,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Volatile").field(&self.read()).finish()
    }
}

impl<T> fmt::Debug for Volatile<T, WriteOnly>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Volatile").field(&"[write-only]").finish()
    }
}

#[macro_export]
macro_rules! map_field {
    ($volatile:ident.$place:ident) => {
        unsafe { $volatile.map(|ptr| core::ptr::addr_of_mut!((*ptr).$place)) }
    };
}

#[macro_export]
macro_rules! map_field_mut {
    ($volatile:ident.$place:ident) => {
        unsafe { $volatile.map_mut(|ptr| core::ptr::addr_of_mut!((*ptr).$place)) }
    };
}

#[cfg(test)]
mod tests {
    use super::Volatile;
    use core::cell::UnsafeCell;

    #[test]
    fn test_read() {
        let val = 42;
        assert_eq!(unsafe { Volatile::from_ptr(&val) }.read(), 42);
    }

    #[test]
    fn test_write() {
        let mut val = 50;
        let mut volatile = unsafe { Volatile::from_mut_ptr(&mut val) };
        volatile.write(50);
        assert_eq!(val, 50);
    }

    #[test]
    fn test_update() {
        let mut val = 42;
        let mut volatile = unsafe { Volatile::from_mut_ptr(&mut val) };
        volatile.update(|v| *v += 1);
        assert_eq!(val, 43);
    }

    #[test]
    fn test_struct() {
        #[derive(Debug, PartialEq)]
        struct S {
            field_1: u32,
            field_2: bool,
        }

        let mut val = S {
            field_1: 60,
            field_2: true,
        };
        let mut volatile = unsafe { Volatile::from_mut_ptr(&mut val) };
        unsafe { volatile.map_mut(|s| core::ptr::addr_of_mut!((*s).field_1)) }.update(|v| *v += 1);
        let mut field_2 = unsafe { volatile.map_mut(|s| core::ptr::addr_of_mut!((*s).field_2)) };
        assert!(field_2.read());
        field_2.write(false);
        assert_eq!(
            val,
            S {
                field_1: 61,
                field_2: false
            }
        );
    }

    #[test]
    fn test_struct_macro() {
        #[derive(Debug, PartialEq)]
        struct S {
            field_1: u32,
            field_2: bool,
        }

        let mut val = S {
            field_1: 60,
            field_2: true,
        };
        let mut volatile = unsafe { Volatile::from_mut_ptr(&mut val) };
        let mut field_1 = map_field_mut!(volatile.field_1);
        field_1.update(|v| *v += 1);
        let mut field_2 = map_field_mut!(volatile.field_2);
        assert!(field_2.read());
        field_2.write(false);
        assert_eq!(
            val,
            S {
                field_1: 61,
                field_2: false
            }
        );
    }

    #[cfg(feature = "unstable")]
    #[test]
    fn test_slice() {
        let mut val: &mut [u32] = &mut [1, 2, 3];
        let mut volatile = Volatile::from_mut_ptr(val);
        volatile.index_mut(0).update(|v| *v += 1);
        assert_eq!(val, [2, 2, 3]);
    }

    #[cfg(feature = "unstable")]
    #[test]
    fn test_chunks() {
        let mut val: &mut [u32] = &mut [1, 2, 3, 4, 5, 6];
        let mut volatile = Volatile::from_mut_ptr(val);
        let mut chunks = volatile.as_chunks_mut().0;
        chunks.index_mut(1).write([10, 11, 12]);
        assert_eq!(chunks.index(0).read(), [1, 2, 3]);
        assert_eq!(chunks.index(1).read(), [10, 11, 12]);
    }
}
