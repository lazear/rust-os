use crate::prelude::*;

pub mod pic;
pub mod pit;

/// Run initialization functions for PIC, PIT, etc
pub fn init() {
    let _ = pic::Intel8259::global().lock();
    let _ = pit::Intel8253::init(10_000);
}
