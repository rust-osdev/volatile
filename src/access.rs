pub trait Access: Copy + Default {
    /// Ensures that this trait cannot be implemented outside of this crate.
    #[doc(hidden)]
    fn _private() -> _Private {
        _Private
    }

    type RestrictShared: Access;
}

/// Helper trait that is implemented by [`ReadWrite`] and [`ReadOnly`].
pub trait Readable: Access {
    /// Ensures that this trait cannot be implemented outside of this crate.
    fn _private() -> _Private {
        _Private
    }
}

/// Helper trait that is implemented by [`ReadWrite`] and [`WriteOnly`].
pub trait Writable: Access {
    /// Ensures that this trait cannot be implemented outside of this crate.
    fn _private() -> _Private {
        _Private
    }
}

/// Zero-sized marker type for allowing both read and write access.
#[derive(Debug, Default, Copy, Clone)]
pub struct ReadWrite;
impl Access for ReadWrite {
    type RestrictShared = ReadOnly;
}
impl Readable for ReadWrite {}
impl Writable for ReadWrite {}

/// Zero-sized marker type for allowing only read access.
#[derive(Debug, Default, Copy, Clone)]
pub struct ReadOnly;
impl Access for ReadOnly {
    type RestrictShared = ReadOnly;
}
impl Readable for ReadOnly {}

/// Zero-sized marker type for allowing only write access.
#[derive(Debug, Default, Copy, Clone)]
pub struct WriteOnly;
impl Access for WriteOnly {
    type RestrictShared = NoAccess;
}
impl Writable for WriteOnly {}

/// Zero-sized marker type that grants no access.
#[derive(Debug, Default, Copy, Clone)]
pub struct NoAccess;
impl Access for NoAccess {
    type RestrictShared = NoAccess;
}

#[non_exhaustive]
#[doc(hidden)]
pub struct _Private;
