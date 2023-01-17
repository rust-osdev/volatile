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

use crate::access::{Access, ReadOnly, ReadWrite, Readable, Writable, WriteOnly};

#[cfg(test)]
mod tests;
#[cfg(feature = "unstable")]
mod unstable;
#[cfg(feature = "very_unstable")]
mod very_unstable;

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

    pub unsafe fn map_mut<F, U>(self, f: F) -> VolatilePtrCopy<'a, U, A>
    where
        F: FnOnce(NonNull<T>) -> NonNull<U>,
        U: ?Sized,
        A: Access,
    {
        unsafe { VolatilePtrCopy::new_restricted(A::default(), f(self.pointer)) }
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

impl<T, A> fmt::Debug for VolatilePtrCopy<'_, T, A>
where
    T: Copy + fmt::Debug + ?Sized,
    A: Readable,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Volatile").field(&self.read()).finish()
    }
}
