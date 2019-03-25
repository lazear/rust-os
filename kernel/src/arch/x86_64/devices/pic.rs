use crate::io::{Io, Port};
use crate::sync;

const PIC1_CMD: u16 = 0x20;
const PIC2_CMD: u16 = 0xA0;
const PIC1_DATA: u16 = 0x21;
const PIC2_DATA: u16 = 0xA2;
const IRQ_SLAVE: u8 = 0x02;
const IRQ_ZERO: u8 = 0x20;

/// We have a global instance of the PIC because there are mutable operations
/// that can be performed on it (masking/unmasking interrupts)
global!(Intel8259);

/// Intel 8259A Programmable Interrupt Controller
pub struct Intel8259 {
    mask: u16,
    data1: Port<u8>,
    data2: Port<u8>,
}

impl Default for Intel8259 {
    fn default() -> Intel8259 {
        let mut data1 = Port::<u8>::new(PIC1_DATA);
        let mut data2 = Port::<u8>::new(PIC2_DATA);
        let mut cmd1 = Port::<u8>::new(PIC1_CMD);
        let mut cmd2 = Port::<u8>::new(PIC2_CMD);

        data1.write(0xFF);
        data2.write(0xFF);

        // Setup master 8259A
        cmd1.write(0x11);

        // set IRQ vector offset
        // 0x20 is IRQ zero
        data1.write(IRQ_ZERO);
        data1.write(1 << IRQ_SLAVE);
        data1.write(0x03);

        // Setup slave 8259A-2
        cmd2.write(0x11);
        data2.write(IRQ_ZERO + 8);
        data2.write(IRQ_SLAVE);
        data2.write(0x03);

        cmd1.write(0x68);
        cmd1.write(0x0A);
        cmd2.write(0x68);
        cmd2.write(0x0A);

        Intel8259 {
            mask: 0xFFFF & !(1 << IRQ_SLAVE),
            data1,
            data2,
        }
    }
}

impl Intel8259 {
    fn set_mask(&mut self, mask: u16) {
        self.mask = mask;
        // write low word
        self.data1.write((mask & 0x00FF) as u8);
        // and high word
        self.data2.write((mask >> 8) as u8);
    }

    pub fn enable_irq(&mut self, irq: u8) {
        self.set_mask(self.mask & !(1 << irq))
    }

    pub fn disable_irq(&mut self, irq: u8) {
        self.set_mask(self.mask | (1 << irq))
    }

    pub fn disable_all(&mut self) {
        self.set_mask(0xFFFF)
    }
}
