use crate::access::{ReadWrite, Readable, Writable};
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
#[derive(Debug, Default, Clone)]
#[repr(transparent)]
pub struct Volatile<T, A = ReadWrite> {
    value: T,
    access: PhantomData<A>,
}

impl<T> Volatile<T> {
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
    pub const fn new(value: T) -> Volatile<T> {
        Volatile {
            value,
            access: PhantomData,
        }
    }
}

impl<T: Copy, A> Volatile<&T, A> {
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
}

impl<T: Copy, A> Volatile<&mut T, A> {
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
        assert_eq!(Volatile::new(&42).read(), 42);
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
