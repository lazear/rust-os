mod port;
mod serial;
mod volatile;

pub use port::Port;
pub use serial::Serial;
pub use volatile::Volatile;

pub trait Io {
    type Value;
    fn read(&self) -> Self::Value;
    fn write(&mut self, src: Self::Value);
}
