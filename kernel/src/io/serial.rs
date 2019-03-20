use super::{Io, Port};
use crate::sync::{Once, Mutex, Global};
use core::fmt;

global!(Serial);

pub struct Serial {
    input: Port<u8>,
    output: Port<u8>,
}


const COM1: u16 = 0x3F8;

impl Default for Serial {
    fn default() -> Serial {
        Port::<u8>::new(COM1 + 1).write(0u8);
        Port::<u8>::new(COM1 + 3).write(0x80u8);
        Port::<u8>::new(COM1 + 0).write(0x03u8);
        Port::<u8>::new(COM1 + 1).write(0u8);
        Port::<u8>::new(COM1 + 3).write(0x03u8);
        Port::<u8>::new(COM1 + 2).write(0xC7u8);
        Port::<u8>::new(COM1 + 4).write(0u8);

        Serial {
            input: Port::new(COM1 + 5),
            output: Port::new(COM1),
        }
    }
}

impl Io for Serial {
    type Value = u8;
    fn read(&self) -> u8 {
        while self.input.read() & 0x1 == 0 {}
        self.output.read()
    }

    fn write(&mut self, src: u8) {
        while self.input.read() & 0x20 == 0 {}
        self.output.write(src)
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.chars().for_each(|ch| self.write(ch as u8));
        Ok(())
    }
}
