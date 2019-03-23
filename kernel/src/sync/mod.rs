mod init;
mod mutex;

pub use init::Once;
pub use mutex::{Mutex, MutexGuard};

//use spin;
//pub use spin::Once;

/// Trait that automatically generates a globl variable wrapping a struct
/// behind a `Once<Mutex<T>>`, along with an associated function for the
/// struct that returns a reference to the wrapping `Mutex`.
///
/// `T` must implement `Default` if the first macro expansion of global!
/// is used. Otherwise a function block must be provided that initializes
/// `T`
pub trait Global {
    fn global<'a>() -> &'a Mutex<Self>;
}

#[macro_export]
macro_rules! global {
    ($T:ty) => {
        static __GLOBAL: $crate::sync::Once<$crate::sync::Mutex<$T>> = $crate::sync::Once::new();
        impl $crate::sync::Global for $T
        where
            $T: core::default::Default,
        {
            #[inline(always)]
            fn global<'a>() -> &'a $crate::sync::Mutex<$T> {
                __GLOBAL.call_once(|| $crate::sync::Mutex::default())
            }
        }
    };

    ($T:ty, $func:block) => {
        static __GLOBAL: $crate::sync::Once<$crate::sync::Mutex<$T>> = $crate::sync::Once::new();
        impl $crate::sync::Global for $T {
            #[inline(always)]
            fn global<'a>() -> &'a crate::sync::Mutex<$T> {
                #[inline(never)]
                fn inner() -> $T {
                    $func
                }

                __GLOBAL.call_once(|| crate::sync::Mutex::new(inner()))
            }
        }
    };
}
