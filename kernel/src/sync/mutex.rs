use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{spin_loop_hint, AtomicBool, Ordering};

/// A synchronization primitive that guarantees mutually exclusive access
/// to the wrapped data. Only one thread may have access at any given time
pub struct Mutex<T: ?Sized> {
    lock: AtomicBool,
    inner: UnsafeCell<T>,
}

/// A [`MutexGuard`] guarantees exclusive access to the data contained in the
/// owning [`Mutex`]. When the [`MutexGuard`] is dropped, the [`Mutex`] will
/// automatically unlock
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    _mutex: &'a Mutex<T>,
}

/// A `CriticalMutexGuard` guarantees that maskable hardware interrupts
/// will not fire while the guard is held. Dropping the `CriticalMutexGuard`
/// first releases the `Mutex` and then enables hardware interrupts
pub struct CriticalMutexGuard<'a, T: ?Sized + 'a> {
    _mutex: &'a Mutex<T>,
}

impl<T> Mutex<T> {
    /// Initialize a new [`Mutex`] wrapping `data`
    pub fn new(data: T) -> Mutex<T> {
        Mutex {
            inner: UnsafeCell::new(data),
            lock: AtomicBool::new(false),
        }
    }

    /// Consume the [`Mutex`], returning the interior data
    ///
    /// Due to Rust's ownership model, this method can only be called when
    /// there are no outstanding [`MutexGuard`]'s with references to this
    /// [`Mutex`]
    ///
    /// # Example
    ///
    /// ```
    /// let m = Mutex::new(10u64);
    /// // limit the lifetime of the `MutexGuard` g
    /// {
    ///     let g = m.lock();
    ///     *g += 10;
    /// }
    /// assert_eq!(m.into_inner(), 20);
    /// ````
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

    /// Release the [`Mutex`]
    ///
    /// # Safety
    ///
    /// Unsafe if the function is called on a thread that does not own the
    /// Mutex's lock
    #[inline(always)]
    unsafe fn release(&self) {
        self.lock.store(false, Ordering::Release);
    }

    /// Forcefully obtain a mutable reference to the [`Mutex`]'s interior data,
    /// regardless of whether the [`Mutex`] is currently locked.
    ///
    /// # Safety
    ///
    /// This function is *very* unsafe, and should only be used in situations
    /// where the data can be safefully accessed during a potential deadlock
    /// i.e. during an interrupt handler within the kernel operating on the
    /// global VGA terminal lock
    pub unsafe fn force(&self) -> &mut T {
        &mut *self.inner.get()
    }

    /// Attempt to lock the `Mutex`, returning a new [`MutexGuard`] if the
    /// [`Mutex`] was successfully locked by the caller.
    ///
    /// This function will not block
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        // If lock was not held we just acquired it
        if self.lock.compare_and_swap(false, true, Ordering::Acquire) == false {
            Some(MutexGuard { _mutex: self })
        } else {
            None
        }
    }

    /// Block until the [`Mutex`] can be locked
    pub fn lock(&self) -> MutexGuard<T> {
        match self.try_lock() {
            Some(guard) => guard,
            None => {
                self.acquire();
                MutexGuard { _mutex: self }
            }
        }
    }

    /// Attempt to lock the [`Mutex`], returning a new [`CriticalMutexGuard`]
    /// if the [`Mutex`] was sucessfully locked by the caller.
    ///
    /// This funciton will not block
    ///
    /// # Safety
    ///
    /// This function will disable interrupts, and then *enable* hardware
    /// interrupts if the [`Mutex`]'s lock cannot be obtained.
    ///
    /// Hardware interrupts will also be enabled with the
    /// [`CriticalMutexGuard`] is dropped - so the `critical` functions
    /// should only be used in contexts where hardware interrupts are
    /// expected
    pub fn try_critical(&self) -> Option<CriticalMutexGuard<T>> {
        crate::arch::interrupts::disable();
        if self.lock.compare_and_swap(false, true, Ordering::Acquire) == false {
            Some(CriticalMutexGuard { _mutex: self })
        } else {
            crate::arch::interrupts::enable();
            None
        }
    }

    /// Block until the [`Mutex`] can be locked -
    pub fn critical(&self) -> CriticalMutexGuard<T> {
        match self.try_critical() {
            Some(guard) => guard,
            None => {
                self.acquire();
                CriticalMutexGuard { _mutex: self }
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

impl<'a, T: ?Sized> Deref for CriticalMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self._mutex.inner.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for CriticalMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self._mutex.inner.get() }
    }
}

impl<'a, T: ?Sized> Drop for CriticalMutexGuard<'a, T> {
    #[inline]
    /// Dropping the `CriticalMutexGuard` releases the lock and enables interrupts
    fn drop(&mut self) {
        unsafe {
            self._mutex.release();
        }
        crate::arch::interrupts::enable();
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
