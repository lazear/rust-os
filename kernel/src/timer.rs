use crate::prelude::*;
use crate::term::Terminal;
pub struct Timer {
    ticks: usize,
    buf: [u8; 80],
}

global!(Timer);

impl Default for Timer {
    fn default() -> Self {
        Timer {
            ticks: 0,
            buf: [' ' as u8; 80],
        }
    }
}

impl Timer {
    pub fn tick(&mut self) {
        self.ticks += 1;
        let mut s = BytesBuf::from_slice(&mut self.buf);
        write!(s, "{}", self.ticks);
        let f = s.as_str().trim();

        Terminal::global().lock().write_at(f, 0, 79 - f.len());

        if self.ticks > 100 {
            panic!("timer!!!");
        }
    }
}

interrupt!(timer, _stack, { Timer::global().lock().tick() });
