use core::ptr;

#[repr(transparent)]
pub struct Volatile<T: Copy>(T);

impl<T: Copy> Volatile<T> {
    pub fn write(&mut self, src: T) {
        unsafe {
            ptr::write_volatile(&mut self.0, src);
        }
    }

    pub fn read(&self) -> T {
        unsafe {
            ptr::read_volatile(&self.0 as *const T)
        }
    }

    pub fn new(data: T) -> Volatile<T> {
        Volatile(data)
    }
}