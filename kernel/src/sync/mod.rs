mod init;
mod mutex;

pub use init::Once;
pub use mutex::{Mutex, MutexGuard};

pub trait Global {
    fn global<'a>() -> &'a Mutex<Self>;
}

#[macro_export]
macro_rules! global {
    ($T:ty) => {
        static __GLOBAL: Once<Mutex<$T>> = Once::new();
        impl Global for $T {
            fn global<'a>() -> &'a Mutex<$T> {
                __GLOBAL.call_once(|| Mutex::default())
            }
        }
    };
}
