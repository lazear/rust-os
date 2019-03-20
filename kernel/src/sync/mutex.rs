use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{spin_loop_hint, AtomicBool, Ordering};

pub struct Mutex<T: ?Sized> {
    lock: AtomicBool,
    inner: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T: ?Sized + 'a> {
    _mutex: &'a Mutex<T>,
}

impl<T> Mutex<T> {
    pub fn new(data: T) -> Mutex<T> {
        Mutex {
            inner: UnsafeCell::new(data),
            lock: AtomicBool::new(false),
        }
    }

    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

impl<T: ?Sized> Mutex<T> {
    /// This function spins until the lock is acquired
    fn acquire(&self) {
        // spin while the lock is held
        while self.lock.compare_and_swap(false, true, Ordering::Acquire) {
            while self.lock.load(Ordering::Relaxed) {
                spin_loop_hint();
            }
        }
    }

    /// Unsafe if the function is called on a thread that does not own the
    /// Mutex's lock
    unsafe fn release(&self) {
        self.lock.store(false, Ordering::Release);
    }

    /// Attempt to lock the `Mutex`
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        // If lock was not held we just acquired it
        if self.lock.compare_and_swap(false, true, Ordering::Acquire) == false {
            Some(MutexGuard { _mutex: self })
        } else {
            None
        }
    }

    /// Block until the `Mutex` can be locked
    pub fn lock(&self) -> MutexGuard<T> {
        match self.try_lock() {
            Some(guard) => guard,
            None => {
                self.acquire();
                MutexGuard { _mutex: self }
            }
        }
    }
}

impl<T> From<T> for Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    /// This is equivalent to [`Mutex::new`].
    fn from(t: T) -> Self {
        Mutex::new(t)
    }
}

impl<T: ?Sized + Default> Default for Mutex<T> {
    /// Creates a `Mutex<T>`, with the `Default` value for T.
    fn default() -> Mutex<T> {
        Mutex::new(Default::default())
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self._mutex.inner.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self._mutex.inner.get() }
    }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    #[inline]
    /// Dropping the `MutexGuard` releases the lock
    fn drop(&mut self) {
        unsafe {
            self._mutex.release();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_lock() {
        let mutex = Mutex::new(42);

        // First lock succeeds
        let a = mutex.try_lock();
        assert_eq!(a.as_ref().map(|r| **r), Some(42));

        // Additional lock failes
        let b = mutex.try_lock();
        assert!(b.is_none());

        // After dropping lock, it succeeds again
        core::mem::drop(a);
        let c = mutex.try_lock();
        assert_eq!(c.as_ref().map(|r| **r), Some(42));
    }
}
