mod init;
mod mutex;

pub use init::Once;
pub use mutex::{Mutex, MutexGuard};

/// Trait that automatically generates a globl variable wrapping a struct
/// behind a `Once<Mutex<T>>`, along with an associated function for the 
/// struct that returns a reference to the wrapping `Mutex`. 
/// 
/// `T` must implement `Default`
pub trait Global {
    fn global<'a>() -> &'a Mutex<Self>;
}

#[macro_export]
macro_rules! global {
    ($T:ty) => {
        static __GLOBAL: Once<Mutex<$T>> = Once::new();
        impl Global for $T {
            #[inline(always)]
            fn global<'a>() -> &'a Mutex<$T> {
                __GLOBAL.call_once(|| Mutex::default())
            }
        }
    };
    ($T:ty, $E:expr) => {
        static __GLOBAL: Once<Mutex<$T>> = Once::new();
        impl Global for $T {
            #[inline(always)]
            fn global<'a>() -> &'a Mutex<$T> {
                __GLOBAL.call_once(|| Mutex::new($E))
            }
        }
    }
}
