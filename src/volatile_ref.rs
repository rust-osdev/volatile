use crate::{
    access::{Access, Copyable, ReadOnly, ReadWrite, Readable, WriteOnly},
    volatile_ptr,
};
use core::{fmt, marker::PhantomData, ptr::NonNull};

/// Volatile pointer type that respects Rust's aliasing rules.
///
/// This pointer type behaves similar to Rust's reference types:
///
/// - it requires exclusive `&mut self` access for mutability
/// - only read-only types implement [`Clone`] and [`Copy`]
/// - [`Send`] and [`Sync`] are implemented if `T: Sync`
///
/// To perform volatile operations on `VolatileRef` types, use the [`as_ptr`][Self::as_ptr]
/// or [`as_mut_ptr`](Self::as_mut_ptr) methods to create a temporary
/// [`VolatilePtr`][crate::VolatilePtr] instance.
///
/// Since not all volatile resources (e.g. memory mapped device registers) are both readable
/// and writable, this type supports limiting the allowed access types through an optional second
/// generic parameter `A` that can be one of `ReadWrite`, `ReadOnly`, or `WriteOnly`. It defaults
/// to `ReadWrite`, which allows all operations.
///
/// The size of this struct is the same as the size of the contained reference.
#[repr(transparent)]
pub struct VolatileRef<'a, T, A = ReadWrite>
where
    T: ?Sized,
{
    pointer: NonNull<T>,
    reference: PhantomData<&'a T>,
    access: PhantomData<A>,
}

/// Constructor functions.
///
/// These functions construct new `VolatileRef` values. While the `new`
/// function creates a `VolatileRef` instance with unrestricted access, there
/// are also functions for creating read-only or write-only instances.
impl<'a, T> VolatileRef<'a, T>
where
    T: ?Sized,
{
    pub unsafe fn new(pointer: NonNull<T>) -> Self {
        unsafe { VolatileRef::new_restricted(ReadWrite, pointer) }
    }

    pub fn from_mut_ref(reference: &'a mut T) -> Self
    where
        T: 'a,
    {
        unsafe { VolatileRef::new(reference.into()) }
    }

    pub const unsafe fn new_read_only(pointer: NonNull<T>) -> VolatileRef<'a, T, ReadOnly> {
        unsafe { Self::new_restricted(ReadOnly, pointer) }
    }

    pub const unsafe fn new_restricted<A>(access: A, pointer: NonNull<T>) -> VolatileRef<'a, T, A>
    where
        A: Access,
    {
        let _ = access;
        unsafe { Self::new_generic(pointer) }
    }

    pub fn from_ref(reference: &'a T) -> VolatileRef<'a, T, ReadOnly>
    where
        T: 'a,
    {
        unsafe { VolatileRef::new_restricted(ReadOnly, reference.into()) }
    }

    const unsafe fn new_generic<A>(pointer: NonNull<T>) -> VolatileRef<'a, T, A> {
        VolatileRef {
            pointer,
            reference: PhantomData,
            access: PhantomData,
        }
    }
}

impl<'a, T, A> VolatileRef<'a, T, A>
where
    T: ?Sized,
{
    pub fn as_ptr(&self) -> volatile_ptr::VolatilePtr<'_, T, A::RestrictShared>
    where
        A: Access,
    {
        unsafe { volatile_ptr::VolatilePtr::new_restricted(Default::default(), self.pointer) }
    }

    pub fn as_mut_ptr(&mut self) -> volatile_ptr::VolatilePtr<'_, T, A>
    where
        A: Access,
    {
        unsafe { volatile_ptr::VolatilePtr::new_restricted(Default::default(), self.pointer) }
    }
}

/// Methods for restricting access.
impl<'a, T> VolatileRef<'a, T, ReadWrite>
where
    T: ?Sized,
{
    /// Restricts access permissions to read-only.
    ///
    /// ## Example
    ///
    /// ```
    /// use volatile::VolatileRef;
    /// use core::ptr::NonNull;
    ///
    /// let mut value: i16 = -4;
    /// let mut volatile = VolatileRef::from_mut_ref(&mut value);
    ///
    /// let read_only = volatile.read_only();
    /// assert_eq!(read_only.as_ptr().read(), -4);
    /// // read_only.as_ptr().write(10); // compile-time error
    /// ```
    pub fn read_only(self) -> VolatileRef<'a, T, ReadOnly> {
        unsafe { VolatileRef::new_restricted(ReadOnly, self.pointer) }
    }

    /// Restricts access permissions to write-only.
    ///
    /// ## Example
    ///
    /// Creating a write-only reference to a struct field:
    ///
    /// ```
    /// use volatile::{VolatileRef};
    /// use core::ptr::NonNull;
    ///
    /// #[derive(Clone, Copy)]
    /// struct Example { field_1: u32, field_2: u8, }
    /// let mut value = Example { field_1: 15, field_2: 255 };
    /// let mut volatile = VolatileRef::from_mut_ref(&mut value);
    ///
    /// let write_only = volatile.write_only();
    /// // write_only.as_ptr().read(); // compile-time error
    /// ```
    pub fn write_only(self) -> VolatileRef<'a, T, WriteOnly> {
        unsafe { VolatileRef::new_restricted(WriteOnly, self.pointer) }
    }
}

impl<'a, T, A> Clone for VolatileRef<'a, T, A>
where
    T: ?Sized,
    A: Access + Copyable,
{
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer,
            reference: self.reference,
            access: self.access,
        }
    }
}

impl<'a, T, A> Copy for VolatileRef<'a, T, A>
where
    T: ?Sized,
    A: Access + Copyable,
{
}

unsafe impl<T, A> Send for VolatileRef<'_, T, A> where T: Sync {}
unsafe impl<T, A> Sync for VolatileRef<'_, T, A> where T: Sync {}

impl<T, A> fmt::Debug for VolatileRef<'_, T, A>
where
    T: Copy + fmt::Debug + ?Sized,
    A: Readable,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Volatile")
            .field(&self.as_ptr().read())
            .finish()
    }
}
