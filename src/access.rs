pub trait Access {}

/// Helper trait that is implemented by [`ReadWrite`] and [`ReadOnly`].
pub trait Readable: UnsafelyReadable {}

/// Helper trait that is implemented by [`ReadWrite`] and [`WriteOnly`].
pub trait Writable: UnsafelyWritable {}

pub trait UnsafelyReadable {}

pub trait UnsafelyWritable {}

/// Zero-sized marker type for allowing both read and write access.
#[derive(Debug, Copy, Clone)]
pub struct ReadWrite;
impl Access for ReadWrite {}
impl Readable for ReadWrite {}
impl UnsafelyReadable for ReadWrite {}
impl Writable for ReadWrite {}
impl UnsafelyWritable for ReadWrite {}

/// Zero-sized marker type for allowing only read access.
#[derive(Debug, Copy, Clone)]
pub struct ReadOnly;

impl Access for ReadOnly {}
impl Readable for ReadOnly {}
impl UnsafelyReadable for ReadOnly {}

/// Zero-sized marker type for allowing only write access.
#[derive(Debug, Copy, Clone)]
pub struct WriteOnly;

impl Access for WriteOnly {}
impl Writable for WriteOnly {}
impl UnsafelyWritable for WriteOnly {}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Custom<R, W> {
    pub read: R,
    pub write: W,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NoAccess;
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SafeAccess;
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UnsafeAccess;

impl<W> Readable for Custom<SafeAccess, W> {}
impl<W> UnsafelyReadable for Custom<SafeAccess, W> {}
impl<W> UnsafelyReadable for Custom<UnsafeAccess, W> {}
impl<R> Writable for Custom<R, SafeAccess> {}
impl<R> UnsafelyWritable for Custom<R, SafeAccess> {}
impl<R> UnsafelyWritable for Custom<R, UnsafeAccess> {}
