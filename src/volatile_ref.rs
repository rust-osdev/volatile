use crate::{
    access::{Access, Copyable, ReadOnly, ReadWrite, RestrictAccess, WriteOnly},
    volatile_ptr::VolatilePtr,
};
use core::{cmp::Ordering, fmt, hash, marker::PhantomData, ptr::NonNull};

/// Volatile pointer type that respects Rust's aliasing rules.
///
/// This pointer type behaves similar to Rust's reference types:
///
/// - it requires exclusive `&mut self` access for mutability
/// - only read-only types implement [`Clone`] and [`Copy`]
/// - [`Send`] and [`Sync`] are implemented if `T: Sync`
///
/// However, trait implementations like [`fmt::Debug`] and [`Eq`] behave like they do on pointer
/// types and don't access the referenced value.
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
#[must_use]
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
    /// Turns the given pointer into a `VolatileRef`.
    ///
    /// ## Safety
    ///
    /// - The pointer must be properly aligned.
    /// - It must be “dereferenceable” in the sense defined in the [`core::ptr`] documentation.
    /// - The pointer must point to an initialized instance of T.
    /// - You must enforce Rust’s aliasing rules, since the returned lifetime 'a is arbitrarily
    ///   chosen and does not necessarily reflect the actual lifetime of the data. In particular,
    ///   while this `VolatileRef` exists, the memory the pointer points to must not get accessed
    ///   (_read or written_) through any other pointer.
    pub const unsafe fn new(pointer: NonNull<T>) -> Self {
        unsafe { VolatileRef::new_restricted(ReadWrite, pointer) }
    }

    /// Turns the given pointer into a read-only `VolatileRef`.
    ///
    /// ## Safety
    ///
    /// - The pointer must be properly aligned.
    /// - It must be “dereferenceable” in the sense defined in the [`core::ptr`] documentation.
    /// - The pointer must point to an initialized instance of T.
    /// - You must enforce Rust’s aliasing rules, since the returned lifetime 'a is arbitrarily
    ///   chosen and does not necessarily reflect the actual lifetime of the data. In particular,
    ///   while this `VolatileRef` exists, the memory the pointer points to _must not get mutated_.
    pub const unsafe fn new_read_only(pointer: NonNull<T>) -> VolatileRef<'a, T, ReadOnly> {
        unsafe { Self::new_restricted(ReadOnly, pointer) }
    }

    /// Turns the given pointer into a `VolatileRef` instance with the given access.
    ///
    /// ## Safety
    ///
    /// - The pointer must be properly aligned.
    /// - It must be “dereferenceable” in the sense defined in the [`core::ptr`] documentation.
    /// - The pointer must point to an initialized instance of T.
    /// - You must enforce Rust’s aliasing rules, since the returned lifetime 'a is arbitrarily
    ///   chosen and does not necessarily reflect the actual lifetime of the data. In particular,
    ///   while this `VolatileRef` exists, the memory the pointer points to _must not get mutated_.
    ///   If the given `access` parameter allows write access, the pointer _must not get read
    ///   either_ while this `VolatileRef` exists.
    pub const unsafe fn new_restricted<A>(access: A, pointer: NonNull<T>) -> VolatileRef<'a, T, A>
    where
        A: Access,
    {
        let _ = access;
        unsafe { Self::new_generic(pointer) }
    }

    /// Creates a `VolatileRef` from the given shared reference.
    ///
    /// **Note:** This function is only intended for testing, not for accessing real volatile
    /// data. The reason is that the `&mut T` argument is considered _dereferenceable_ by Rust,
    /// so the compiler is allowed to insert non-volatile reads. This might lead to undesired
    /// (or even undefined?) behavior when accessing volatile data. So to be safe, only create
    /// raw pointers to volatile data and use the [`Self::new`] constructor instead.
    pub const fn from_ref(reference: &'a T) -> VolatileRef<'a, T, ReadOnly>
    where
        T: 'a,
    {
        unsafe { VolatileRef::new_restricted(ReadOnly, NonNull::from_ref(reference)) }
    }

    /// Creates a `VolatileRef` from the given mutable reference.
    ///
    /// **Note:** This function is only intended for testing, not for accessing real volatile
    /// data. The reason is that the `&mut T` argument is considered _dereferenceable_ by Rust,
    /// so the compiler is allowed to insert non-volatile reads. This might lead to undesired
    /// (or even undefined?) behavior when accessing volatile data. So to be safe, only create
    /// raw pointers to volatile data and use the [`Self::new`] constructor instead.
    pub const fn from_mut_ref(reference: &'a mut T) -> Self
    where
        T: 'a,
    {
        unsafe { VolatileRef::new(NonNull::from_ref(reference)) }
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
    /// Immutably borrows from this `VolatileRef`.
    ///
    /// This method creates a `VolatileRef` tied to the lifetime of the `&VolatileRef` it is created from.
    /// This is useful for providing a volatile reference without moving the original `VolatileRef`.
    /// In comparison with creating a `&VolatileRef<'a, T>`, this avoids the additional indirection and lifetime.
    pub fn borrow(&self) -> VolatileRef<'_, T, A::Restricted>
    where
        A: RestrictAccess<ReadOnly>,
    {
        unsafe { VolatileRef::new_restricted(Default::default(), self.pointer) }
    }

    /// Mutably borrows from this `VolatileRef`.
    ///
    /// This method creates a `VolatileRef` tied to the lifetime of the `&mut VolatileRef` it is created from.
    /// This is useful for providing a volatile reference without moving the original `VolatileRef`.
    /// In comparison with creating a `&mut VolatileRef<'a, T>`, this avoids the additional indirection and lifetime.
    pub fn borrow_mut(&mut self) -> VolatileRef<'_, T, A>
    where
        A: Access,
    {
        unsafe { VolatileRef::new_restricted(Default::default(), self.pointer) }
    }

    /// Borrows this `VolatileRef` as a read-only [`VolatilePtr`].
    ///
    /// Use this method to do (partial) volatile reads of the referenced data.
    pub fn as_ptr(&self) -> VolatilePtr<'_, T, A::Restricted>
    where
        A: RestrictAccess<ReadOnly>,
    {
        unsafe { VolatilePtr::new_restricted(Default::default(), self.pointer) }
    }

    /// Borrows this `VolatileRef` as a mutable [`VolatilePtr`].
    ///
    /// Use this method to do (partial) volatile reads or writes of the referenced data.
    pub fn as_mut_ptr(&mut self) -> VolatilePtr<'_, T, A>
    where
        A: Access,
    {
        unsafe { VolatilePtr::new_restricted(Default::default(), self.pointer) }
    }

    /// Converts this `VolatileRef` into a [`VolatilePtr`] with full access without shortening
    /// the lifetime.
    ///
    /// Use this method when you need a [`VolatilePtr`] instance that lives for the full
    /// lifetime `'a`.
    ///
    /// This method consumes the `VolatileRef`.
    pub fn into_ptr(self) -> VolatilePtr<'a, T, A>
    where
        A: Access,
    {
        unsafe { VolatilePtr::new_restricted(Default::default(), self.pointer) }
    }
}

/// Methods for restricting access.
impl<'a, T, A> VolatileRef<'a, T, A>
where
    T: ?Sized,
{
    /// Restricts access permissions to `A`.
    ///
    /// ## Example
    ///
    /// ```
    /// use volatile::access::{ReadOnly, WriteOnly};
    /// use volatile::VolatileRef;
    ///
    /// let mut value: i16 = -4;
    /// let volatile = VolatileRef::from_mut_ref(&mut value);
    ///
    /// let read_only = volatile.restrict::<ReadOnly>();
    /// assert_eq!(read_only.as_ptr().read(), -4);
    /// // read_only.as_ptr().write(10); // compile-time error
    ///
    /// let no_access = read_only.restrict::<WriteOnly>();
    /// // no_access.read(); // compile-time error
    /// // no_access.write(10); // compile-time error
    /// ```
    pub fn restrict<To>(self) -> VolatileRef<'a, T, A::Restricted>
    where
        A: RestrictAccess<To>,
    {
        unsafe { VolatileRef::new_restricted(Default::default(), self.pointer) }
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
    ///
    /// let mut value: i16 = -4;
    /// let volatile = VolatileRef::from_mut_ref(&mut value);
    ///
    /// let read_only = volatile.read_only();
    /// assert_eq!(read_only.as_ptr().read(), -4);
    /// // read_only.as_ptr().write(10); // compile-time error
    /// ```
    pub fn read_only(self) -> VolatileRef<'a, T, ReadOnly> {
        self.restrict()
    }

    /// Restricts access permissions to write-only.
    ///
    /// ## Example
    ///
    /// Creating a write-only reference to a struct field:
    ///
    /// ```
    /// use volatile::{VolatileRef};
    ///
    /// #[derive(Clone, Copy)]
    /// struct Example { field_1: u32, field_2: u8, }
    /// let mut value = Example { field_1: 15, field_2: 255 };
    /// let volatile = VolatileRef::from_mut_ref(&mut value);
    ///
    /// let write_only = volatile.write_only();
    /// // write_only.as_ptr().read(); // compile-time error
    /// ```
    pub fn write_only(self) -> VolatileRef<'a, T, WriteOnly> {
        self.restrict()
    }
}

impl<'a, T, A> Clone for VolatileRef<'a, T, A>
where
    T: ?Sized,
    A: Access + Copyable,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T, A> Copy for VolatileRef<'a, T, A>
where
    T: ?Sized,
    A: Access + Copyable,
{
}

unsafe impl<T, A> Send for VolatileRef<'_, T, A> where T: Sync + ?Sized {}
unsafe impl<T, A> Sync for VolatileRef<'_, T, A> where T: Sync + ?Sized {}

impl<T, A> fmt::Debug for VolatileRef<'_, T, A>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.pointer.as_ptr(), f)
    }
}

impl<T, A> fmt::Pointer for VolatileRef<'_, T, A>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.pointer.as_ptr(), f)
    }
}

impl<T, A> PartialEq for VolatileRef<'_, T, A>
where
    T: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.pointer.as_ptr(), other.pointer.as_ptr())
    }
}

impl<T, A> Eq for VolatileRef<'_, T, A> where T: ?Sized {}

impl<T, A> PartialOrd for VolatileRef<'_, T, A>
where
    T: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, A> Ord for VolatileRef<'_, T, A>
where
    T: ?Sized,
{
    fn cmp(&self, other: &Self) -> Ordering {
        #[allow(ambiguous_wide_pointer_comparisons)]
        Ord::cmp(&self.pointer.as_ptr(), &other.pointer.as_ptr())
    }
}

impl<T, A> hash::Hash for VolatileRef<'_, T, A>
where
    T: ?Sized,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.pointer.as_ptr().hash(state);
    }
}
