# Unreleased

- Add implementations for `fmt::Pointer`, `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash`.
- Update `very_unstable` feature to latest nightly
- Remove `Sized` requirement for `Send` and `Sync` impls on `VolatileRef`

# 0.5.1 – 2023-06-24

- Fix: Add missing documentation of the `map` macro

# 0.5.0 – 2023-06-24

- **Breaking:** [New design based on raw pointers](https://github.com/rust-osdev/volatile/pull/29)
  - The previous reference-based design was [unsound](https://github.com/rust-osdev/volatile/pull/13#issuecomment-842455552) because it allowed the compiler to insert spurious reads.
  - The new design features two wrapper types for raw pointers: `VolatilePtr` and `VolatileRef`
  - `VolatilePtr` provides safe read and write access to volatile values. Like raw pointers, it implements `Copy` and is `!Sync`.
  - `VolatileRef` is a pointer type that respects Rust's aliasing rules. It doesn't implement `Copy`, requires a `&mut` reference for modification, and implements `Sync`. It can converted to temporary `VolatilePtr` instances through the `as_ptr`/`as_mut_ptr` methods.
- We now provide methods for volatile slice operations and a `map!` macro for struct field projection. These advanced features are gated behind a cargo feature named _"unstable"_.

# 0.4.6 – 2023-01-17

- Fix UB in slice methods when Deref returns different references ([#27](https://github.com/rust-osdev/volatile/pull/27))

# 0.4.5 – 2022-04-24

- Remove the `const_generics` feature flag ([#25](https://github.com/rust-osdev/volatile/pull/25))

# 0.4.4 – 2021-03-09

- Replace feature "range_bounds_assert_len" with "slice_range" ([#21](https://github.com/rust-osdev/volatile/pull/21))
  - Fixes the `unstable` feature on the latest nightly.

# 0.4.3 – 2020-12-23

- Add methods to restrict access ([#19](https://github.com/rust-osdev/volatile/pull/19))

# 0.4.2 – 2020-10-31

- Change `slice::check_range` to `RangeBounds::assert_len` ([#16](https://github.com/rust-osdev/volatile/pull/16))
  - Fixes build on latest nightly.

# 0.4.1 – 2020-09-21

- Small documentation and metadata improvements

# 0.4.0 – 2020-09-21

- **Breaking:** Rewrite crate to operate on reference values ([#13](https://github.com/rust-osdev/volatile/pull/13))

# 0.3.0 – 2020-07-29

- **Breaking:** Remove `Debug` and `Clone` derives for `WriteOnly` ([#12](https://github.com/rust-osdev/volatile/pull/12))

# 0.2.7 – 2020-07-29

- Derive `Default` for `Volatile`, `WriteOnly` and `ReadOnly` ([#10](https://github.com/embed-rs/volatile/pull/10))
