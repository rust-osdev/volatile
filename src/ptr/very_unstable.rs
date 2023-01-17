use core::ptr::NonNull;

use crate::{access::Access, VolatilePtr};

impl<'a, T, A> VolatilePtr<'a, T, A>
where
    T: ?Sized,
{
    pub const unsafe fn map_const<F, U>(self, f: F) -> VolatilePtr<'a, U, A::RestrictShared>
    where
        F: ~const FnOnce(NonNull<T>) -> NonNull<U>,
        A: Access,
        U: ?Sized,
    {
        unsafe { VolatilePtr::new_generic(f(self.pointer)) }
    }

    pub const unsafe fn map_mut_const<F, U>(self, f: F) -> VolatilePtr<'a, U, A>
    where
        F: ~const FnOnce(NonNull<T>) -> NonNull<U>,
        U: ?Sized,
    {
        unsafe { VolatilePtr::new_generic(f(self.pointer)) }
    }
}

/// Methods for volatile slices
#[cfg(feature = "unstable")]
impl<'a, T, A> VolatilePtr<'a, [T], A> {
    pub const fn index_const(self, index: usize) -> VolatilePtr<'a, T, A::RestrictShared>
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

    pub const fn index_mut_const(self, index: usize) -> VolatilePtr<'a, T, A> {
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
}
