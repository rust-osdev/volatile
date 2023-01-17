#![no_std]

pub use cell::VolatileCell;

pub mod access;
mod cell;
mod ptr;
