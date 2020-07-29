use core::{ops::Index, ptr, slice::SliceIndex};

/// A wrapper type around a volatile variable, which allows for volatile reads and writes
/// to the contained value. The stored type needs to be `Copy`, as volatile reads and writes
/// take and return copies of the value.
///
/// The size of this struct is the same as the size of the contained type.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct VolatileRef<'a, T: ?Sized> {
    pub(crate) value: &'a T,
}

impl<T> VolatileRef<'_, T> {
    /// Construct a new volatile instance wrapping the given value.
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let value = Volatile::new(0u32);
    /// ```
    ///
    /// # Panics///
    /// This method never panics.
    #[cfg(feature = "const_fn")]
    pub const fn new(value: &T) -> VolatileRef<T> {
        Volatile {
            value,
            access: PhantomData,
        }
    }

    /// Construct a new volatile instance wrapping the given value.
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let value = Volatile::new(0u32);
    /// ```
    ///
    /// # Panics
    ///
    /// This method never panics.
    #[cfg(not(feature = "const_fn"))]
    pub fn new(value: &T) -> VolatileRef<T> {
        VolatileRef { value }
    }
}

impl<T: Copy> VolatileRef<'_, T> {
    /// Performs a volatile read of the contained value, returning a copy
    /// of the read value. Volatile reads are guaranteed not to be optimized
    /// away by the compiler, but by themselves do not have atomic ordering
    /// guarantees. To also get atomicity, consider looking at the `Atomic` wrapper type.
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let value = Volatile::new(42u32);
    ///
    /// assert_eq!(value.read(), 42u32);
    /// ```
    ///
    /// # Panics
    ///
    /// This method never panics.
    pub fn read(&self) -> T {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::read_volatile(self.value) }
    }
}

impl<T> VolatileRef<'_, [T]> {
    pub fn index<I>(&self, index: I) -> VolatileRef<I::Output>
    where
        I: SliceIndex<[T], Output = [T]>,
    {
        VolatileRef {
            value: self.value.index(index),
        }
    }
}
