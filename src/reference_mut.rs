use crate::{
    access::{ReadWrite, Readable, Writable},
    reference::VolatileRef,
};
use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr,
    slice::SliceIndex,
};

/// A wrapper type around a volatile variable, which allows for volatile reads and writes
/// to the contained value. The stored type needs to be `Copy`, as volatile reads and writes
/// take and return copies of the value.
///
/// The size of this struct is the same as the size of the contained type.
#[derive(Debug)]
#[repr(transparent)]
pub struct VolatileRefMut<'a, T: ?Sized, A = ReadWrite> {
    value: &'a mut T,
    access: PhantomData<A>,
}

impl<T> VolatileRefMut<'_, T> {
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
    #[cfg(feature = "const_fn")]
    pub const fn new(value: &mut T) -> VolatileRefMut<T> {
        VolatileRefMut {
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
    pub fn new(value: &mut T) -> VolatileRefMut<T> {
        VolatileRefMut {
            value,
            access: PhantomData,
        }
    }
}

impl<T: Copy, A> VolatileRefMut<'_, T, A> {
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
    pub fn read(&self) -> T
    where
        A: Readable,
    {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::read_volatile(self.value) }
    }

    /// Performs a volatile write, setting the contained value to the given value `value`. Volatile
    /// writes are guaranteed to not be optimized away by the compiler, but by themselves do not
    /// have atomic ordering guarantees. To also get atomicity, consider looking at the `Atomic`
    /// wrapper type.
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let mut value = Volatile::new(0u32);
    ///
    /// value.write(42u32);
    ///
    /// assert_eq!(value.read(), 42u32);
    /// ```
    ///
    /// # Panics
    ///
    /// This method never panics.
    pub fn write(&mut self, value: T)
    where
        A: Writable,
    {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::write_volatile(self.value, value) };
    }

    /// Performs a volatile read of the contained value, passes a mutable reference to it to the
    /// function `f`, and then performs a volatile write of the (potentially updated) value back to
    /// the contained value.
    ///
    /// ```rust
    /// use volatile::Volatile;
    ///
    /// let mut value = Volatile::new(21u32);
    ///
    /// value.update(|val_ref| *val_ref *= 2);
    ///
    /// assert_eq!(value.read(), 42u32);
    /// ```
    ///
    /// # Panics
    ///
    /// Ths method never panics.
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

impl<T, A> VolatileRefMut<'_, [T], A> {
    pub fn index<I>(&self, index: I) -> VolatileRef<I::Output>
    where
        I: SliceIndex<[T], Output = [T]>,
        A: Readable,
    {
        VolatileRef {
            value: self.value.index(index),
        }
    }

    pub fn index_mut<I>(&mut self, index: I) -> VolatileRefMut<I::Output, A>
    where
        I: SliceIndex<[T], Output = [T]>,
    {
        VolatileRefMut {
            value: self.value.index_mut(index),
            access: self.access,
        }
    }
}
