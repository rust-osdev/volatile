# Unreleased

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
