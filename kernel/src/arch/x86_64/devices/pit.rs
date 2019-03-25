use crate::io::{Io, Port};
use crate::sync::*;

/// Intel 8253 programmable interval timer
pub struct Intel8253;

static INIT: Once<()> = Once::new();

impl Intel8253 {
    pub fn init(frequency: u32) {
        INIT.call_once(|| {
            let divisor: u32 = 1193180 / frequency;

            let mut cmd = Port::<u8>::new(0x43);
            let mut data = Port::<u8>::new(0x40);

            cmd.write(0x34);
            cmd.read();
            data.write((divisor & 0xFF) as u8);
            data.read();
            data.write((divisor >> 8) as u8);
        });
    }
}
