mod port;
mod volatile;
mod serial;

pub use port::Port;
pub use volatile::Volatile;
pub use serial::Serial;

pub trait Io {
    type Value;
    fn read(&self) -> Self::Value;
    fn write(&mut self, src: Self::Value);
}
