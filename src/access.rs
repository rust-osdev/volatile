pub trait Access: Copy + Default {
    /// Ensures that this trait cannot be implemented outside of this crate.
    #[doc(hidden)]
    fn _private() -> _Private {
        _Private
    }
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
impl Access for ReadWrite {}
impl Readable for ReadWrite {}
impl Writable for ReadWrite {}

/// Zero-sized marker type for allowing only read access.
#[derive(Debug, Default, Copy, Clone)]
pub struct ReadOnly;
impl Access for ReadOnly {}
impl Readable for ReadOnly {}

/// Zero-sized marker type for allowing only write access.
#[derive(Debug, Default, Copy, Clone)]
pub struct WriteOnly;
impl Access for WriteOnly {}
impl Writable for WriteOnly {}

#[non_exhaustive]
#[doc(hidden)]
pub struct _Private;
