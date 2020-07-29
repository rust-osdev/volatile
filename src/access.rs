pub trait Readable {}
pub trait Writable {}

pub struct ReadWrite;
impl Readable for ReadWrite {}
impl Writable for ReadWrite {}

pub struct ReadOnly;

impl Readable for ReadOnly {}

pub struct WriteOnly;
impl Writable for WriteOnly {}
