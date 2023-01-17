#![no_std]
#![cfg_attr(feature = "unstable", feature(core_intrinsics))]
#![cfg_attr(feature = "unstable", feature(slice_range))]
#![cfg_attr(feature = "unstable", feature(slice_ptr_get))]
#![cfg_attr(feature = "unstable", feature(slice_ptr_len))]
#![cfg_attr(feature = "very_unstable", feature(const_slice_ptr_len))]
#![cfg_attr(feature = "very_unstable", feature(const_panic))]
#![cfg_attr(feature = "very_unstable", feature(const_fn_trait_bound))]
#![cfg_attr(feature = "very_unstable", feature(const_fn_fn_ptr_basics))]
#![cfg_attr(feature = "very_unstable", feature(const_trait_impl))]
#![cfg_attr(feature = "very_unstable", feature(const_mut_refs))]
#![cfg_attr(feature = "very_unstable", allow(incomplete_features))]
#![cfg_attr(all(feature = "unstable", test), feature(slice_as_chunks))]
#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod access;
mod cell;
mod ptr;
