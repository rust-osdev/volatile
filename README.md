# `volatile`

[![Build Status](https://github.com/rust-osdev/volatile/workflows/Build/badge.svg)](https://github.com/rust-osdev/volatile/actions?query=workflow%3ABuild) [![Docs.rs Badge](https://docs.rs/volatile/badge.svg)](https://docs.rs/volatile/)

Provides volatile wrapper types for raw pointers.

The volatile wrapper types in this crate wrap a pointer to any `Copy`-able type and provide volatile memory access to wrapped value.
Volatile memory accesses are never optimized away by the compiler, and are useful in many low-level systems programming and concurrent contexts.

This crate provides two different wrapper types: `VolatilePtr` and `VolatileRef`.
The difference between the two types is that the former behaves like a raw pointer, while the latter behaves like a Rust reference type.
For example, `VolatilePtr` can be freely copied, but not sent across threads because this could introduce mutable aliasing.
The `VolatileRef` type, on the other hand, requires exclusive access for mutation, so that sharing it across thread boundaries is safe.

Both wrapper types *do not* enforce any atomicity guarantees; to also get atomicity, consider looking at the `Atomic` wrapper types found in `libcore` or `libstd`.

## Why is there no `VolatileCell`?

Many people expressed interest in a `VolatileCell` type, i.e. a transparent wrapper type that owns the wrapped value.
Such a type would be similar to `core::cell::Cell`, with the difference that all methods are volatile.
Unfortunately, it is not sound to implement such a `VolatileCell` type in Rust.
The reason is that Rust and LLVM consider `&` and `&mut` references as _dereferencable_.
This means that the compiler is allowed to freely access the referenced value without any restrictions.
So no matter how a `VolatileCell` type is implemented, the compiler is allowed to perform non-volatile read operations of the contained value, which can lead to unexpected (or even undefined?) behavior.
For more details, see the discussion [in our repository](https://github.com/rust-osdev/volatile/issues/31) and [in the `unsafe-code-guidelines` repository](https://github.com/rust-lang/unsafe-code-guidelines/issues/411).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
