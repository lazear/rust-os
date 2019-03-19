use super::Io;
use core::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct Port<T> {
    port: u16,
    p: PhantomData<T>,
}

impl<T> Port<T> {
    pub const fn new(port: u16) -> Port<T> {
        Port {
            port,
            p: PhantomData,
        }
    }
}

macro_rules! port_decl {
    ($tt:ty) => {
        impl Io for Port<$tt> {
            type Value = $tt;
            fn read(&self) -> Self::Value {
                let val: Self::Value;
                unsafe {
                    asm!("in $0, $1" : "={ax}"(val) : "{dx}"(self.port) : "memory" : "intel" : "volatile");
                }
                val
            }

            fn write(&mut self, src: Self::Value) {
                unsafe {
                    asm!("out $1, $0" : : "{ax}"(src), "{dx}"(self.port) : "memory" : "intel" : "volatile");
                }
            }
        }
    };
}


port_decl!(u8);
port_decl!(u16);
port_decl!(u32);