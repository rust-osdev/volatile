#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NoAccess;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnsafeAccess;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SafeAccess;

pub trait Unsafe {}
pub trait Safe: Unsafe {}

impl Unsafe for UnsafeAccess {}
impl Unsafe for SafeAccess {}
impl Safe for SafeAccess {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Access<R, W> {
    pub read: R,
    pub write: W,
}

impl Access<SafeAccess, NoAccess> {
    pub const fn read_only() -> ReadOnly {
        Access {
            read: SafeAccess,
            write: NoAccess,
        }
    }

    pub fn write_only() -> WriteOnly {
        Access {
            read: NoAccess,
            write: SafeAccess,
        }
    }

    pub fn read_write() -> ReadWrite {
        Access {
            read: SafeAccess,
            write: SafeAccess,
        }
    }
}

pub type ReadOnly = Access<SafeAccess, NoAccess>;
pub type WriteOnly = Access<NoAccess, SafeAccess>;
pub type ReadWrite = Access<SafeAccess, SafeAccess>;
