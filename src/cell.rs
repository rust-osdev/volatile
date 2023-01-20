// ! Provides the wrapper type `VolatileCell`, which wraps any copy-able type and allows for
// ! volatile memory access to wrapped value. Volatile memory accesses are never optimized away by
// ! the compiler, and are useful in many low-level systems programming and concurrent contexts.
// !
// ! # Dealing with Volatile Pointers
// !
// ! Frequently, one may have to deal with volatile pointers, eg, writes to specific memory
// ! locations. The canonical way to solve this is to cast the pointer to a volatile wrapper
// ! directly, eg:
// !
// ! ```rust
// ! use volatile::VolatileCell;
// !
// ! let mut_ptr = 0xFEE00000 as *mut u32;
// !
// ! let volatile_ptr = mut_ptr as *mut VolatileCell<u32>;
// ! ```
// !
// ! and then perform operations on the pointer as usual in a volatile way. This method works as all
// ! of the volatile wrapper types are the same size as their contained values.

use crate::{
    access::{Access, ReadOnly, ReadWrite, Readable, Writable},
    ptr_send::VolatilePtr,
};
use core::{cell::UnsafeCell, fmt, marker::PhantomData, ptr::NonNull};

/// A wrapper type around a volatile variable, which allows for volatile reads and writes
/// to the contained value. The stored type needs to be `Copy`, as volatile reads and writes
/// take and return copies of the value.
///
/// Volatile operations instruct the compiler to skip certain optimizations for these
/// operations. For example, the compiler will not optimize them away even if it thinks
/// that the operations have no observable effect. This is for example desirable when
/// the value is stored in a special memory region that has side effects, such as
/// memory-mapped device registers.
///
/// Note that this wrapper types *does not* enforce any atomicity guarantees. To get atomicity,
/// use the [`core::sync::atomic`] module.
///
/// The size of this struct is the same as the size of the contained type.
#[derive(Default)]
#[repr(transparent)]
pub struct VolatileCell<T, A = ReadWrite> {
    value: UnsafeCell<T>,
    access: PhantomData<A>,
}

impl<T> VolatileCell<T> {
    /// Construct a new volatile cell wrapping the given value.
    ///
    /// The returned cell allows read and write operations. Use
    /// [`new_restricted`][VolatileCell::new_restricted] to create read-only
    /// or write-only cells.
    ///
    /// Calling `VolatileCell::new(v)` is equivalent to calling
    /// `VolatileCell::new_restricted(access::ReadWrite, v)`.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use volatile::VolatileCell;
    ///
    /// let mut value = VolatileCell::new(0u32);
    /// assert_eq!(value.read(), 0);
    /// value.write(42);
    /// assert_eq!(value.read(), 42);
    /// value.update(|v| v + 2 );
    /// assert_eq!(value.read(), 44);
    /// ```
    pub const fn new(value: T) -> Self {
        VolatileCell::new_restricted(ReadWrite, value)
    }

    /// Construct a new volatile cell with restricted access, wrapping the given value.
    ///
    /// ## Examples
    ///
    /// ```
    /// use volatile::{VolatileCell, access};
    ///
    /// let mut read_write = VolatileCell::new_restricted(access::ReadWrite, 0u32);
    /// read_write.write(100);
    /// read_write.update(|v| v / 2);
    /// assert_eq!(read_write.read(), 50);
    ///
    /// let read_only = VolatileCell::new_restricted(access::ReadOnly, 0u32);
    /// assert_eq!(read_only.read(), 0);
    ///
    /// let mut write_only = VolatileCell::new_restricted(access::WriteOnly, 0u32);
    /// write_only.write(1);
    /// ```
    ///
    /// ```compile_fail
    /// # use volatile::{VolatileCell, access};
    /// // reading or updating a write-only value is not allowed
    /// let write_only = VolatileCell::new_restricted(access::WriteOnly, 0u32);
    /// write_only.read(); // -> compile error
    /// write_only.update(|v| v + 1); // -> compile error
    /// ```
    ///
    /// ```compile_fail
    /// # use volatile::{VolatileCell, access};
    /// // writing or updating a write-only value is not allowed
    /// let read_only = VolatileCell::new_restricted(access::ReadOnly, 0u32);
    /// read_only.write(5); // -> compile error
    /// read_only.update(|v| v + 1); // -> compile error
    /// ```
    pub const fn new_restricted<A>(access: A, value: T) -> VolatileCell<T, A>
    where
        A: Access,
    {
        let _ = access;
        VolatileCell {
            value: UnsafeCell::new(value),
            access: PhantomData,
        }
    }
}

impl<T, A> VolatileCell<T, A> {
    pub fn access(&self) -> A
    where
        A: Access,
    {
        A::default()
    }

    pub fn as_ptr(&self) -> VolatilePtr<T, ReadOnly> {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe {
            VolatilePtr::new_restricted(
                ReadOnly,
                NonNull::new_unchecked(UnsafeCell::raw_get(&self.value)),
            )
        }
    }

    pub fn as_mut_ptr(&mut self) -> VolatilePtr<T, A>
    where
        A: Access,
    {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe {
            VolatilePtr::new_restricted(
                A::default(),
                NonNull::new_unchecked(UnsafeCell::raw_get(&self.value)),
            )
        }
    }

    /// Performs a volatile read of the contained value, returning a copy
    /// of the read value. Volatile reads are guaranteed not to be optimized
    /// away by the compiler, but by themselves do not have atomic ordering
    /// guarantees. To also get atomicity, consider looking at the `Atomic` wrapper type.
    ///
    /// ```rust
    /// use volatile::VolatileCell;
    ///
    /// let value = VolatileCell::new(42u32);
    /// assert_eq!(value.read(), 42u32);
    /// ```
    pub fn read(&self) -> T
    where
        A: Readable,
        T: Copy,
    {
        self.as_ptr().read()
    }

    /// Performs a volatile write, setting the contained value to the given value `value`. Volatile
    /// writes are guaranteed to not be optimized away by the compiler, but by themselves do not
    /// have atomic ordering guarantees. To also get atomicity, consider looking at the `Atomic`
    /// wrapper type.
    ///
    /// ```rust
    /// use volatile::VolatileCell;
    ///
    /// let mut value = VolatileCell::new(0u32);
    /// value.write(42u32);
    /// assert_eq!(value.read(), 42u32);
    /// ```
    pub fn write(&mut self, value: T)
    where
        A: Writable,
        T: Copy,
    {
        self.as_mut_ptr().write(value)
    }

    /// Performs a volatile read of the contained value, passes a mutable reference to it to the
    /// function `f`, and then performs a volatile write of the (potentially updated) value back to
    /// the contained value.
    ///
    /// ```rust
    /// use volatile::VolatileCell;
    ///
    /// let mut value = VolatileCell::new(21u32);
    /// value.update(|val| val * 2);
    /// assert_eq!(value.read(), 42u32);
    /// ```
    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(T) -> T,
        A: Readable + Writable,
        T: Copy,
    {
        let new = f(self.read());
        self.write(new);
    }
}

/// Create a clone of the `VolatileCell`.
///
/// A `VolatileCell` is clonable only if the cell is marked as readable.
///
/// Note that using a `VolatileCell` only makes sense if the backing memory is
/// actually volatile. Stack memory is not volatile normally, so this clone
/// implementation is not needed in most situations. Instead, it is recommended
/// to read out the wrapped value instead.
///
/// Cloning a `VolatileCell` is equivalent to:
///
/// ```rust
/// # use volatile::VolatileCell;
/// # let volatile_cell = VolatileCell::new(0u32);
/// VolatileCell::new_restricted(volatile_cell.access(), volatile_cell.read())
/// # ;
/// ```
impl<T, A> Clone for VolatileCell<T, A>
where
    T: Copy,
    A: Readable,
{
    fn clone(&self) -> Self {
        VolatileCell::new_restricted(self.access(), self.read())
    }
}

/// This `Debug` implementation only applies to cells that are [`Readable`]
/// because it includes the wrapped value.
impl<T, A> fmt::Debug for VolatileCell<T, A>
where
    T: Copy + fmt::Debug,
    A: Readable,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VolatileCell").field(&self.read()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::VolatileCell;

    #[test]
    fn test_read() {
        assert_eq!(VolatileCell::new(42).read(), 42);
    }

    #[test]
    fn test_write() {
        let mut volatile = VolatileCell::new(42);
        volatile.write(50);
        assert_eq!(*volatile.value.get_mut(), 50);
    }

    #[test]
    fn test_update() {
        let mut volatile = VolatileCell::new(42);
        volatile.update(|v| v + 1);
        assert_eq!(volatile.read(), 43);
    }

    #[test]
    fn test_pointer_recast() {
        let mut target_value = 0u32;

        let target_ptr: *mut u32 = &mut target_value;
        let volatile_ptr = target_ptr as *mut VolatileCell<u32>;

        // UNSAFE: Safe, as we know the value exists on the stack.
        unsafe {
            (*volatile_ptr).write(42u32);
        }

        assert_eq!(target_value, 42u32);
    }
}
