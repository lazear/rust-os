use crate::sync::*;
use crate::io::{Port, Io};

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

pub struct Timer {
    ticks: usize,
}

global!(Timer);

impl Default for Timer {
    fn default() -> Self {
        Timer {
            ticks: 0
        }
    }
}

impl Timer {
    pub fn tick(&mut self) {
        self.ticks += 1;

        unsafe {
            let n = (0xB8000 as *mut u8).offset(158);
            *n = ((self.ticks & 0xFF) as u8) + '0' as u8;
        }
    }
}