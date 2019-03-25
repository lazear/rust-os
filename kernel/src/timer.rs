use crate::prelude::*;

pub struct Timer {
    ticks: usize,
}

global!(Timer);

impl Default for Timer {
    fn default() -> Self {
        Timer { ticks: 0 }
    }
}

impl Timer {
    pub fn tick(&mut self) {
        self.ticks += 1;

        unsafe {
            let n = (0xB8000 as *mut u8).offset(158);
            *n = ((self.ticks & 0xFF) as u8) + b'0' as u8;
        }
    }
}

interrupt!(timer, _stack, { Timer::global().lock().tick() });
