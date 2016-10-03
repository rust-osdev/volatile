#![no_std]

use core::ptr;

#[derive(Debug)]
pub struct Volatile<T: Copy>(T);

impl<T: Copy> Volatile<T> {
    pub fn read(&self) -> T {
        unsafe { ptr::read_volatile(&self.0) }
    }

    pub fn write(&mut self, value: T) {
        unsafe { ptr::write_volatile(&mut self.0, value) };
    }

    pub fn update<F>(&mut self, f: F)
        where F: FnOnce(&mut T)
    {
        let mut value = self.read();
        f(&mut value);
        self.write(value);
    }
}

#[cfg(test)]
mod tests {
    use super::Volatile;

    #[test]
    fn test_read() {
        assert_eq!(Volatile(42).read(), 42);
    }

    #[test]
    fn test_write() {
        let mut volatile = Volatile(42);
        volatile.write(50);
        assert_eq!(volatile.0, 50);
    }

    #[test]
    fn test_update() {
        let mut volatile = Volatile(42);
        volatile.update(|v| *v += 1);
        assert_eq!(volatile.0, 43);
    }
}
