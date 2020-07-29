pub trait Readable {}
pub trait Writable {}

pub struct Read;

impl Readable for Read {}

pub struct Write;
impl Writable for Write {}

pub struct ReadWrite;
impl Readable for ReadWrite {}
impl Writable for ReadWrite {}
