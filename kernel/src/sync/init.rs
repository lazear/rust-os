use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Once<T> {
    state: AtomicBool,
    inner: UnsafeCell<Option<T>>,
}

unsafe impl<T: Send + Sync> Sync for Once<T> {}
unsafe impl<T: Send> Send for Once<T> {}

impl<T> Once<T> {
    pub const fn new() -> Once<T> {
        Once {
            state: AtomicBool::new(false),
            inner: UnsafeCell::new(None),
        }
    }

    unsafe fn get<'a>(&'a self) -> &'a T {
        match (*self.inner.get()).as_ref() {
            Some(ptr) => ptr,
            None => unreachable!(),
        }
    }

    pub fn call_once<'a, F: FnOnce() -> T>(&'a self, func: F) -> &'a T {
        if !self.state.compare_and_swap(false, true, Ordering::Acquire) {
            unsafe {
                *self.inner.get() = Some(func());
            }
        }
        unsafe { self.get() }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn invalid_access() {
        let init = Once::new();
        unsafe {
            assert_eq!(*init.get(), 10);
        }
        init.call_once(|| 10);
    }

    #[test]
    fn multiple() {
        let init = Once::new();
        init.call_once(|| 10);
        assert_eq!(init.call_once(|| 12), &10);
        assert_eq!(init.call_once(|| 22), &10);
        assert_eq!(init.call_once(|| 32), &10);
    }
}
