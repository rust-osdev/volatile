//! Provides the wrapper type `Volatile`, which wraps a reference to any copy-able type and allows
//! for volatile memory access to wrapped value. Volatile memory accesses are never optimized away
//! by the compiler, and are useful in many low-level systems programming and concurrent contexts.
//!
//! The wrapper types *do not* enforce any atomicity guarantees; to also get atomicity, consider
//! looking at the `Atomic` wrapper types found in `libcore` or `libstd`.

use core::{
    fmt,
    marker::PhantomData,
    ptr::{self, NonNull},
};
#[cfg(feature = "unstable")]
use core::{
    intrinsics,
    ops::{Range, RangeBounds},
    slice::{range, SliceIndex},
};

use crate::access::{Access, ReadOnly, ReadWrite, Readable, Writable, WriteOnly};

/// Wraps a pointer to make accesses to the referenced value volatile.
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
pub struct VolatilePtrCopy<'a, T, A = ReadWrite>
where
    T: ?Sized,
{
    pointer: NonNull<T>,
    reference: PhantomData<&'a T>,
    access: PhantomData<A>,
}

impl<'a, T, A> Copy for VolatilePtrCopy<'a, T, A> where T: ?Sized {}

impl<T, A> Clone for VolatilePtrCopy<'_, T, A>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        *self
    }
}

/// Constructor functions.
///
/// These functions construct new `VolatilePtr` values. While the `new`
/// function creates a `VolatilePtr` instance with unrestricted access, there
/// are also functions for creating read-only or write-only instances.
impl<'a, T> VolatilePtrCopy<'a, T>
where
    T: ?Sized,
{
    pub unsafe fn new(pointer: NonNull<T>) -> Self {
        unsafe { VolatilePtrCopy::new_restricted(ReadWrite, pointer) }
    }

    pub fn from_mut_ref(reference: &'a mut T) -> Self
    where
        T: 'a,
    {
        unsafe { VolatilePtrCopy::new(reference.into()) }
    }

    pub const unsafe fn new_read_only(pointer: NonNull<T>) -> VolatilePtrCopy<'a, T, ReadOnly> {
        unsafe { Self::new_restricted(ReadOnly, pointer) }
    }

    pub const unsafe fn new_restricted<A>(
        access: A,
        pointer: NonNull<T>,
    ) -> VolatilePtrCopy<'a, T, A>
    where
        A: Access,
    {
        let _ = access;
        unsafe { Self::new_generic(pointer) }
    }

    pub fn from_ref(reference: &'a T) -> VolatilePtrCopy<'a, T, ReadOnly>
    where
        T: 'a,
    {
        unsafe { VolatilePtrCopy::new_restricted(ReadOnly, reference.into()) }
    }

    const unsafe fn new_generic<A>(pointer: NonNull<T>) -> VolatilePtrCopy<'a, T, A> {
        VolatilePtrCopy {
            pointer,
            reference: PhantomData,
            access: PhantomData,
        }
    }
}

impl<'a, T, A> VolatilePtrCopy<'a, T, A>
where
    T: ?Sized,
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
    /// use volatile::{VolatilePtrCopy, access};
    /// use core::ptr::NonNull;
    ///
    /// let value = 42;
    /// let shared_reference = unsafe {
    ///     VolatilePtrCopy::new_restricted(access::ReadOnly, NonNull::from(&value))
    /// };
    /// assert_eq!(shared_reference.read(), 42);
    ///
    /// let mut value = 50;
    /// let mut_reference = VolatilePtrCopy::from_mut_ref(&mut value);
    /// assert_eq!(mut_reference.read(), 50);
    /// ```
    pub fn read(self) -> T
    where
        T: Copy,
        A: Readable,
    {
        // UNSAFE: Safe, as ... TODO
        unsafe { ptr::read_volatile(self.pointer.as_ptr()) }
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
    /// use volatile::VolatilePtrCopy;
    /// use core::ptr::NonNull;
    ///
    /// let mut value = 42;
    /// let mut volatile = VolatilePtrCopy::from_mut_ref(&mut value);
    /// volatile.write(50);
    ///
    /// assert_eq!(volatile.read(), 50);
    /// ```
    pub fn write(self, value: T)
    where
        T: Copy,
        A: Writable,
    {
        // UNSAFE: Safe, as ... TODO
        unsafe { ptr::write_volatile(self.pointer.as_ptr(), value) };
    }

    /// Updates the contained value using the given closure and volatile instructions.
    ///
    /// Performs a volatile read of the contained value, passes a mutable reference to it to the
    /// function `f`, and then performs a volatile write of the (potentially updated) value back to
    /// the contained value.
    ///
    /// ```rust
    /// use volatile::VolatilePtrCopy;
    /// use core::ptr::NonNull;
    ///
    /// let mut value = 42;
    /// let mut volatile = VolatilePtrCopy::from_mut_ref(&mut value);
    /// volatile.update(|val| val + 1);
    ///
    /// assert_eq!(volatile.read(), 43);
    /// ```
    pub fn update<F>(self, f: F)
    where
        T: Copy,
        A: Readable + Writable,
        F: FnOnce(T) -> T,
    {
        let new = f(self.read());
        self.write(new);
    }

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
    /// use volatile::VolatilePtrCopy;
    /// use core::ptr::NonNull;
    ///
    /// let mut value = 42;
    /// let mut volatile = VolatilePtrCopy::from_mut_ref(&mut value);
    /// volatile.write(50);
    /// let unwrapped: *mut i32 = volatile.as_ptr().as_ptr();
    ///
    /// assert_eq!(unsafe { *unwrapped }, 50); // non volatile access, be careful!
    /// ```
    pub fn as_ptr(self) -> NonNull<T> {
        self.pointer
    }

    /// Constructs a new `Volatile` reference by mapping the wrapped pointer.
    ///
    /// This method is useful for accessing only a part of a volatile value, e.g. a subslice or
    /// a struct field. For struct field access, there is also the safe [`map_field`] macro that
    /// wraps this function.
    ///
    /// ## Examples
    ///
    /// Accessing a struct field:
    ///
    /// ```
    /// use volatile::VolatilePtrCopy;
    /// use core::ptr::NonNull;
    ///
    /// struct Example { field_1: u32, field_2: u8, }
    /// let mut value = Example { field_1: 15, field_2: 255 };
    /// let mut volatile = VolatilePtrCopy::from_mut_ref(&mut value);
    ///
    /// // construct a volatile reference to a field
    /// let field_2 = unsafe { volatile.map(|ptr| NonNull::new(core::ptr::addr_of_mut!((*ptr.as_ptr()).field_2)).unwrap()) };
    /// assert_eq!(field_2.read(), 255);
    /// ```
    ///
    /// Don't misuse this method to do a non-volatile read of the referenced value:
    ///
    /// ```
    /// use volatile::VolatilePtrCopy;
    /// use core::ptr::NonNull;
    ///
    /// let mut value = 5;
    /// let mut volatile = VolatilePtrCopy::from_mut_ref(&mut value);
    ///
    /// // DON'T DO THIS:
    /// let mut readout = 0;
    /// unsafe { volatile.map(|value| {
    ///    readout = *value.as_ptr(); // non-volatile read, might lead to bugs
    ///    value
    /// })};
    /// ```
    pub unsafe fn map<F, U>(self, f: F) -> VolatilePtrCopy<'a, U, A::RestrictShared>
    where
        F: FnOnce(NonNull<T>) -> NonNull<U>,
        A: Access,
        U: ?Sized,
    {
        unsafe { VolatilePtrCopy::new_restricted(Default::default(), f(self.pointer)) }
    }

    #[cfg(feature = "very_unstable")]
    pub const unsafe fn map_const<F, U>(self, f: F) -> VolatilePtrCopy<'a, U, A::RestrictShared>
    where
        F: ~const FnOnce(NonNull<T>) -> NonNull<U>,
        A: Access,
        U: ?Sized,
    {
        unsafe { VolatilePtrCopy::new_generic(f(self.pointer)) }
    }

    pub unsafe fn map_mut<F, U>(self, f: F) -> VolatilePtrCopy<'a, U, A>
    where
        F: FnOnce(NonNull<T>) -> NonNull<U>,
        U: ?Sized,
        A: Access,
    {
        unsafe { VolatilePtrCopy::new_restricted(A::default(), f(self.pointer)) }
    }

    #[cfg(feature = "very_unstable")]
    pub const unsafe fn map_mut_const<F, U>(self, f: F) -> VolatilePtrCopy<'a, U, A>
    where
        F: ~const FnOnce(NonNull<T>) -> NonNull<U>,
        U: ?Sized,
    {
        unsafe { VolatilePtrCopy::new_generic(f(self.pointer)) }
    }
}

/// Methods for restricting access.
impl<'a, T> VolatilePtrCopy<'a, T, ReadWrite>
where
    T: ?Sized,
{
    /// Restricts access permissions to read-only.
    ///
    /// ## Example
    ///
    /// ```
    /// use volatile::VolatilePtrCopy;
    /// use core::ptr::NonNull;
    ///
    /// let mut value: i16 = -4;
    /// let mut volatile = VolatilePtrCopy::from_mut_ref(&mut value);
    ///
    /// let read_only = volatile.read_only();
    /// assert_eq!(read_only.read(), -4);
    /// // read_only.write(10); // compile-time error
    /// ```
    pub fn read_only(self) -> VolatilePtrCopy<'a, T, ReadOnly> {
        unsafe { VolatilePtrCopy::new_restricted(ReadOnly, self.pointer) }
    }

    /// Restricts access permissions to write-only.
    ///
    /// ## Example
    ///
    /// Creating a write-only reference to a struct field:
    ///
    /// ```
    /// use volatile::{VolatilePtrCopy, map_field_mut};
    /// use core::ptr::NonNull;
    ///
    /// struct Example { field_1: u32, field_2: u8, }
    /// let mut value = Example { field_1: 15, field_2: 255 };
    /// let mut volatile = VolatilePtrCopy::from_mut_ref(&mut value);
    ///
    /// // construct a volatile write-only reference to `field_2`
    /// let mut field_2 = map_field_mut!(volatile.field_2).write_only();
    /// field_2.write(14);
    /// // field_2.read(); // compile-time error
    /// ```
    pub fn write_only(self) -> VolatilePtrCopy<'a, T, WriteOnly> {
        unsafe { VolatilePtrCopy::new_restricted(WriteOnly, self.pointer) }
    }
}

/// Methods for volatile slices
#[cfg(feature = "unstable")]
impl<'a, T, A> VolatilePtrCopy<'a, [T], A> {
    pub fn len(self) -> usize {
        self.pointer.len()
    }

    pub fn is_empty(self) -> bool {
        self.pointer.len() == 0
    }

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
    /// use volatile::VolatilePtrCopy;
    /// use core::ptr::NonNull;
    ///
    /// let array = [1, 2, 3];
    /// let slice = &array[..];
    /// let volatile = unsafe { VolatilePtrCopy::new_read_only(NonNull::from(slice)) };
    /// assert_eq!(volatile.index(1).read(), 2);
    /// ```
    ///
    /// Accessing a subslice:
    ///
    /// ```
    /// use volatile::VolatilePtrCopy;
    /// use core::ptr::NonNull;
    ///
    /// let array = [1, 2, 3];
    /// let slice = &array[..];
    /// let volatile = unsafe { VolatilePtrCopy::new_read_only(NonNull::from(slice)) };
    /// let subslice = volatile.index(1..);
    /// assert_eq!(subslice.index(0).read(), 2);
    /// ```
    pub fn index<I>(
        self,
        index: I,
    ) -> VolatilePtrCopy<'a, <I as SliceIndex<[T]>>::Output, A::RestrictShared>
    where
        I: SliceIndex<[T]> + SliceIndex<[()]> + Clone,
        A: Access,
    {
        bounds_check(self.pointer.len(), index.clone());

        unsafe { self.map(|slice| slice.get_unchecked_mut(index)) }
    }

    #[cfg(feature = "very_unstable")]
    pub const fn index_const(self, index: usize) -> VolatilePtrCopy<'a, T, A::RestrictShared>
    where
        A: Access,
    {
        assert!(index < self.pointer.len(), "index out of bounds");

        struct Mapper {
            index: usize,
        }
        impl<T> const FnOnce<(NonNull<[T]>,)> for Mapper {
            type Output = NonNull<T>;

            extern "rust-call" fn call_once(self, (slice,): (NonNull<[T]>,)) -> Self::Output {
                unsafe { NonNull::new_unchecked(slice.as_non_null_ptr().as_ptr().add(self.index)) }
            }
        }

        unsafe { self.map_const(Mapper { index }) }
    }

    pub fn index_mut<I>(self, index: I) -> VolatilePtrCopy<'a, <I as SliceIndex<[T]>>::Output, A>
    where
        I: SliceIndex<[T]> + SliceIndex<[()]> + Clone,
        A: Access,
    {
        bounds_check(self.pointer.len(), index.clone());

        unsafe { self.map_mut(|slice| slice.get_unchecked_mut(index)) }
    }

    #[cfg(feature = "very_unstable")]
    pub const fn index_mut_const(self, index: usize) -> VolatilePtrCopy<'a, T, A> {
        assert!(index < self.pointer.len(), "index out of bounds");

        struct Mapper {
            index: usize,
        }
        impl<T> const FnOnce<(NonNull<[T]>,)> for Mapper {
            type Output = NonNull<T>;

            extern "rust-call" fn call_once(self, (slice,): (NonNull<[T]>,)) -> Self::Output {
                unsafe { NonNull::new_unchecked(slice.as_non_null_ptr().as_ptr().add(self.index)) }
            }
        }

        unsafe { self.map_mut_const(Mapper { index }) }
    }

    /// Returns an iterator over the slice.
    pub fn iter(self) -> impl Iterator<Item = VolatilePtrCopy<'a, T, A::RestrictShared>>
    where
        A: Access,
    {
        let ptr = self.as_ptr().as_ptr() as *mut T;
        let len = self.len();
        (0..len).map(move |i| unsafe {
            VolatilePtrCopy::new_generic(NonNull::new_unchecked(ptr.add(i)))
        })
    }

    /// Returns an iterator that allows modifying each value.
    pub fn iter_mut(self) -> impl Iterator<Item = VolatilePtrCopy<'a, T, A>> {
        let ptr = self.as_ptr().as_ptr() as *mut T;
        let len = self.len();
        (0..len).map(move |i| unsafe {
            VolatilePtrCopy::new_generic(NonNull::new_unchecked(ptr.add(i)))
        })
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
    /// use volatile::VolatilePtrCopy;
    /// use core::ptr::NonNull;
    ///
    /// let src = [1, 2];
    /// // the `Volatile` type does not work with arrays, so convert `src` to a slice
    /// let slice = &src[..];
    /// let volatile = unsafe { VolatilePtrCopy::new_read_only(NonNull::from(slice)) };
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
    pub fn copy_into_slice(self, dst: &mut [T])
    where
        T: Copy,
        A: Readable,
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
    /// use volatile::VolatilePtrCopy;
    /// use core::ptr::NonNull;
    ///
    /// let src = [1, 2, 3, 4];
    /// let mut dst = [0, 0];
    /// // the `Volatile` type does not work with arrays, so convert `dst` to a slice
    /// let slice = &mut dst[..];
    /// let mut volatile = VolatilePtrCopy::from_mut_ref(slice);
    ///    /// // Because the slices have to be the same length,
    /// // we slice the source slice from four elements
    /// // to two. It will panic if we don't do this.
    /// volatile.copy_from_slice(&src[2..]);
    ///
    /// assert_eq!(src, [1, 2, 3, 4]);
    /// assert_eq!(dst, [3, 4]);
    /// ```
    pub fn copy_from_slice(self, src: &[T])
    where
        T: Copy,
        A: Writable,
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
    /// extern crate core;
    /// use volatile::VolatilePtrCopy;
    /// use core::ptr::NonNull;
    ///
    /// let mut byte_array = *b"Hello, World!";
    /// let mut slice: &mut [u8] = &mut byte_array[..];
    /// let mut volatile = VolatilePtrCopy::from_mut_ref(slice);
    /// volatile.copy_within(1..5, 8);
    ///
    /// assert_eq!(&byte_array, b"Hello, Wello!");
    pub fn copy_within(self, src: impl RangeBounds<usize>, dest: usize)
    where
        T: Copy,
        A: Readable + Writable,
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

    pub fn split_at(
        self,
        mid: usize,
    ) -> (
        VolatilePtrCopy<'a, [T], A::RestrictShared>,
        VolatilePtrCopy<'a, [T], A::RestrictShared>,
    )
    where
        A: Access,
    {
        assert!(mid <= self.pointer.len());
        // SAFETY: `[ptr; mid]` and `[mid; len]` are inside `self`, which
        // fulfills the requirements of `from_raw_parts_mut`.
        unsafe { self.split_at_unchecked(mid) }
    }

    pub fn split_at_mut(
        self,
        mid: usize,
    ) -> (VolatilePtrCopy<'a, [T], A>, VolatilePtrCopy<'a, [T], A>)
    where
        A: Access,
    {
        assert!(mid <= self.pointer.len());
        // SAFETY: `[ptr; mid]` and `[mid; len]` are inside `self`, which
        // fulfills the requirements of `from_raw_parts_mut`.
        unsafe { self.split_at_mut_unchecked(mid) }
    }

    unsafe fn split_at_unchecked(
        self,
        mid: usize,
    ) -> (
        VolatilePtrCopy<'a, [T], A::RestrictShared>,
        VolatilePtrCopy<'a, [T], A::RestrictShared>,
    )
    where
        A: Access,
    {
        // SAFETY: Caller has to check that `0 <= mid <= self.len()`
        unsafe {
            (
                VolatilePtrCopy::new_generic((self.pointer).get_unchecked_mut(..mid)),
                VolatilePtrCopy::new_generic((self.pointer).get_unchecked_mut(mid..)),
            )
        }
    }

    unsafe fn split_at_mut_unchecked(
        self,
        mid: usize,
    ) -> (VolatilePtrCopy<'a, [T], A>, VolatilePtrCopy<'a, [T], A>)
    where
        A: Access,
    {
        let len = self.pointer.len();
        let ptr = self.pointer.as_mut_ptr();

        // SAFETY: Caller has to check that `0 <= mid <= self.len()`.
        //
        // `[ptr; mid]` and `[mid; len]` are not overlapping, so returning a mutable reference
        // is fine.
        unsafe {
            (
                VolatilePtrCopy::new_generic(
                    NonNull::new(ptr::slice_from_raw_parts_mut(ptr, mid)).unwrap(),
                ),
                VolatilePtrCopy::new_generic(
                    NonNull::new(ptr::slice_from_raw_parts_mut(ptr.add(mid), len - mid)).unwrap(),
                ),
            )
        }
    }

    pub fn as_chunks<const N: usize>(
        self,
    ) -> (
        VolatilePtrCopy<'a, [[T; N]], <A::RestrictShared as Access>::RestrictShared>,
        VolatilePtrCopy<'a, [T], A::RestrictShared>,
    )
    where
        A: Access,
    {
        assert_ne!(N, 0);
        let len = self.pointer.len() / N;
        let (multiple_of_n, remainder) = self.split_at(len * N);
        // SAFETY: We already panicked for zero, and ensured by construction
        // that the length of the subslice is a multiple of N.
        let array_slice = unsafe { multiple_of_n.as_chunks_unchecked() };
        (array_slice, remainder)
    }

    pub unsafe fn as_chunks_unchecked<const N: usize>(
        self,
    ) -> VolatilePtrCopy<'a, [[T; N]], A::RestrictShared>
    where
        A: Access,
    {
        debug_assert_ne!(N, 0);
        debug_assert_eq!(self.pointer.len() % N, 0);
        let new_len =
            // SAFETY: Our precondition is exactly what's needed to call this
            unsafe { core::intrinsics::exact_div(self.pointer.len(), N) };
        // SAFETY: We cast a slice of `new_len * N` elements into
        // a slice of `new_len` many `N` elements chunks.
        let pointer = NonNull::new(ptr::slice_from_raw_parts_mut(
            self.pointer.as_mut_ptr().cast(),
            new_len,
        ))
        .unwrap();
        unsafe { VolatilePtrCopy::new_generic(pointer) }
    }

    pub fn as_chunks_mut<const N: usize>(
        self,
    ) -> (
        VolatilePtrCopy<'a, [[T; N]], A>,
        VolatilePtrCopy<'a, [T], A>,
    )
    where
        A: Access,
    {
        assert_ne!(N, 0);
        let len = self.pointer.len() / N;
        let (multiple_of_n, remainder) = self.split_at_mut(len * N);
        // SAFETY: We already panicked for zero, and ensured by construction
        // that the length of the subslice is a multiple of N.
        let array_slice = unsafe { multiple_of_n.as_chunks_unchecked_mut() };
        (array_slice, remainder)
    }

    pub unsafe fn as_chunks_unchecked_mut<const N: usize>(
        self,
    ) -> VolatilePtrCopy<'a, [[T; N]], A> {
        debug_assert_ne!(N, 0);
        debug_assert_eq!(self.pointer.len() % N, 0);
        let new_len =
            // SAFETY: Our precondition is exactly what's needed to call this
            unsafe { core::intrinsics::exact_div(self.pointer.len(), N) };
        // SAFETY: We cast a slice of `new_len * N` elements into
        // a slice of `new_len` many `N` elements chunks.
        let pointer = NonNull::new(ptr::slice_from_raw_parts_mut(
            self.pointer.as_mut_ptr().cast(),
            new_len,
        ))
        .unwrap();
        unsafe { VolatilePtrCopy::new_generic(pointer) }
    }
}

/// Methods for volatile byte slices
#[cfg(feature = "unstable")]
impl<A> VolatilePtrCopy<'_, [u8], A> {
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
    /// use volatile::VolatilePtrCopy;
    /// use core::ptr::NonNull;
    ///
    /// let mut vec = vec![0; 10];
    /// let mut buf = VolatilePtrCopy::from_mut_ref(vec.as_mut_slice());
    /// buf.fill(1);
    /// assert_eq!(unsafe { buf.as_ptr().as_mut() }, &mut vec![1; 10]);
    /// ```
    pub fn fill(self, value: u8)
    where
        A: Writable,
    {
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
impl<'a, T, A, const N: usize> VolatilePtrCopy<'a, [T; N], A> {
    /// Converts an array reference to a shared slice.
    ///
    /// This makes it possible to use the methods defined on slices.
    ///
    /// ## Example
    ///
    /// Copying two elements from a volatile array reference using `copy_into_slice`:
    ///
    /// ```
    /// use volatile::VolatilePtrCopy;
    /// use core::ptr::NonNull;
    ///
    /// let src = [1, 2];
    /// let volatile = unsafe { VolatilePtrCopy::new_read_only(NonNull::from(&src)) };
    /// let mut dst = [0, 0];
    ///
    /// // convert the `Volatile<&[i32; 2]>` array reference to a `Volatile<&[i32]>` slice
    /// let volatile_slice = volatile.as_slice();
    /// // we can now use the slice methods
    /// volatile_slice.copy_into_slice(&mut dst);
    ///
    /// assert_eq!(dst, [1, 2]);
    /// ```
    pub fn as_slice(self) -> VolatilePtrCopy<'a, [T], A::RestrictShared>
    where
        A: Access,
    {
        unsafe {
            self.map(|array| {
                NonNull::new(ptr::slice_from_raw_parts_mut(array.as_ptr() as *mut T, N)).unwrap()
            })
        }
    }

    /// Converts an array reference to a shared slice.
    ///
    /// This makes it possible to use the methods defined on slices.
    ///
    /// ## Example
    ///
    /// Copying two elements into a volatile array reference using `copy_from_slice`:
    ///
    /// ```
    /// use volatile::{access, VolatilePtrCopy};
    /// use core::ptr::NonNull;
    ///
    /// let src = [1, 2];
    /// let mut dst = [0, 0];
    /// let mut volatile = unsafe { VolatilePtrCopy::new_restricted(access::WriteOnly, NonNull::from(&dst)) };
    ///
    /// // convert the `Volatile<[i32; 2]>` array reference to a `Volatile<[i32]>` slice
    /// let mut volatile_slice = volatile.as_slice_mut();
    /// // we can now use the slice methods
    /// volatile_slice.copy_from_slice(&src);
    ///
    /// assert_eq!(dst, [1, 2]);
    /// ```
    pub fn as_slice_mut(self) -> VolatilePtrCopy<'a, [T], A>
    where
        A: Access,
    {
        unsafe {
            self.map_mut(|array| {
                NonNull::new(ptr::slice_from_raw_parts_mut(array.as_ptr() as *mut T, N)).unwrap()
            })
        }
    }
}

impl<T, A> fmt::Debug for VolatilePtrCopy<'_, T, A>
where
    T: Copy + fmt::Debug + ?Sized,
    A: Readable,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Volatile").field(&self.read()).finish()
    }
}

#[cfg(feature = "unstable")]
fn bounds_check(len: usize, index: impl SliceIndex<[()]>) {
    const MAX_ARRAY: [(); usize::MAX] = [(); usize::MAX];

    let bound_check_slice = &MAX_ARRAY[..len];
    let _ = &bound_check_slice[index];
}