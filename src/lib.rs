//! Provides volatile wrapper types for raw pointers.
//!
//! The volatile wrapper types in this crate wrap a pointer to any [`Copy`]-able
//! type and provide volatile memory access to wrapped value. Volatile memory accesses are
//! never optimized away by the compiler, and are useful in many low-level systems programming
//! and concurrent contexts.
//!
//! This crate provides two different wrapper types: [`VolatilePtr`] and [`VolatileRef`]. The
//! difference between the two types is that the former behaves like a raw pointer, while the
//! latter behaves like a Rust reference type. For example, `VolatilePtr` can be freely copied,
//! but not sent across threads because this could introduce mutable aliasing. The `VolatileRef`
//! type, on the other hand, requires exclusive access for mutation, so that sharing it across
//! thread boundaries is safe.
//!
//! Both wrapper types *do not* enforce any atomicity guarantees; to also get atomicity, consider
//! looking at the `Atomic` wrapper types found in `libcore` or `libstd`.

#![no_std]
#![cfg_attr(feature = "unstable", feature(core_intrinsics))]
#![cfg_attr(feature = "unstable", feature(slice_range))]
#![cfg_attr(feature = "unstable", feature(slice_ptr_get))]
#![cfg_attr(feature = "unstable", feature(slice_ptr_len))]
#![cfg_attr(feature = "very_unstable", feature(const_slice_ptr_len))]
#![cfg_attr(feature = "very_unstable", feature(const_trait_impl))]
#![cfg_attr(feature = "very_unstable", feature(const_mut_refs))]
#![cfg_attr(feature = "very_unstable", feature(inline_const))]
#![cfg_attr(feature = "very_unstable", feature(unboxed_closures))]
#![cfg_attr(feature = "very_unstable", feature(fn_traits))]
#![cfg_attr(all(feature = "unstable", test), feature(slice_as_chunks))]
#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

pub use volatile_ptr::VolatilePtr;
pub use volatile_ref::VolatileRef;

pub mod access;
mod macros;
mod volatile_ptr;
mod volatile_ref;
