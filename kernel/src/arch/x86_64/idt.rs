use core::marker::PhantomData;
use core::u16;
use crate::prelude::*;
use super::PrivilegeLevel;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Entry<P> {
    offset_low: u16,
    segment_selector: u16,
    ty: EntryType,
    offset_mid: u16,
    offset_high: u32,
    _res: u32,
    _phantom: PhantomData<P>,
}

impl<P> Entry<P> {
    pub const fn empty() -> Entry<P> {
        Entry {
            offset_low: 0,
            segment_selector: 0,
            ty: EntryType::new(),
            offset_mid: 0,
            offset_high: 0,
            _res: 0,
            _phantom: PhantomData,
        }
    }

    fn set_handler(&mut self, addr: u64) {
        self.offset_low = addr as u16;
        self.offset_mid = (addr >> 16) as u16;
        self.offset_high = (addr >> 32) as u32;
        self.segment_selector = 0x20;
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct EntryType(u16);

impl EntryType {
    /// Create a new Interrupt Descriptor Table Entry bitflags
    /// with the required bits set to 1 and all other bits set 
    /// to 0. This initialized a new IDT gate with 32 bit flags
    pub const fn new() -> EntryType {
        // Bit 15 - present
        // Bit 13..14 - descriptor privilege level
        // Bits 12 - set to 0 for interrupt and trap gates
        // Bits 8..11 - IDT gate type
        // Bits 1..7 - must be 0
        EntryType(0b0000_1110_0000_0000)
    }

    pub fn set_present(&mut self, present: bool) {
        self.0.set_bit(15, present);
    }

    pub fn set_privilege(&mut self, privilege: PrivilegeLevel) {
        self.0.set_bits(13..14, privilege as u16);
    }

    pub fn set_interrupt(&mut self, enable: bool) {
        self.0.set_bit(8, enable);
    }
}