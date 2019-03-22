use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering, spin_loop_hint};

pub struct Once<T> {
    state: AtomicUsize,
    inner: UnsafeCell<Option<T>>,
}

unsafe impl<T: Send + Sync> Sync for Once<T> {}
unsafe impl<T: Send> Send for Once<T> {}

const EMPTY: usize = 0;
const RUNNING: usize = 1;
const FINISH: usize = 2;

impl<T> Once<T> {
    pub const fn new() -> Once<T> {
        Once {
            state: AtomicUsize::new(EMPTY),
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
        let mut state = self.state.load(Ordering::SeqCst);
        if state == EMPTY {
            state = self.state.compare_and_swap(EMPTY, RUNNING, Ordering::SeqCst);
            if state == EMPTY {
                unsafe{ *self.inner.get() = Some(func()));
                state = FINISH;
                self.state.store(state, Ordering::SeqCst);
                return unsafe { self.get() }; 
            }
        }
        loop {
            match state {
                FINISH => return unsafe { self.get() },
                RUNNING => {
                 //   spin_loop_hint();
                    state = self.state.load(Ordering::SeqCst);
                },
                _ => unreachable!()
            }
        }
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
