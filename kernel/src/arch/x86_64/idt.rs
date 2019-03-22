use super::{interrupts, PrivilegeLevel};
use crate::prelude::*;
use core::mem;
use core::u16;

global!(InterruptDescriptorTable);

#[repr(align(32))]
#[repr(C)]
pub struct InterruptDescriptorTable {
    entries: [Entry; 256],
}

impl Default for InterruptDescriptorTable {
    fn default() -> InterruptDescriptorTable {
        let mut idt = InterruptDescriptorTable::new();
        idt.entries[0x00].set_handler(interrupts::divide_by_zero);
        idt.entries[0x01].set_handler(interrupts::debug);
        idt.entries[0x02].set_handler(interrupts::nonmaskable);
        idt.entries[0x03].set_handler(interrupts::breakpoint);
        idt.entries[0x04].set_handler(interrupts::overflow);
        idt.entries[0x05].set_handler(interrupts::bound_range);
        idt.entries[0x06].set_handler(interrupts::invalid_opcode);
        idt.entries[0x07].set_handler(interrupts::device_not_available);
        idt.entries[0x08].set_handler(interrupts::double_fault);
        idt.entries[0x09].set_handler(interrupts::coprocessor_segment);
        idt.entries[0x0A].set_handler(interrupts::invalid_tss);
        idt.entries[0x0B].set_handler(interrupts::segment_not_present);
        idt.entries[0x0C].set_handler(interrupts::stack_segment);
        idt.entries[0x0D].set_handler(interrupts::protection);
        idt.entries[0x0E].set_handler(interrupts::page);
        idt.entries[0x10].set_handler(interrupts::fpu);
        idt.entries[0x11].set_handler(interrupts::alignment_check);
        idt.entries[0x12].set_handler(interrupts::machine_check);
        idt.entries[0x13].set_handler(interrupts::simd);
        idt
    }
}

impl InterruptDescriptorTable {
    pub const fn new() -> InterruptDescriptorTable {
        InterruptDescriptorTable {
            entries: [Entry::empty(); 256],
        }
    }

    pub fn load(&self) {
        let ptr = crate::arch::DescriptorTablePtr {
            base: self as *const _ as usize,
            limit: (mem::size_of::<Self>() - 1) as u16,
        };
        unsafe {
            asm!("lidt ($0)" :: "r"(&ptr) : "memory" );
        }
    }

    pub fn entry(&mut self, index: u8) -> &mut Entry {
        &mut self.entries[index as usize]
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Entry {
    pub offset_low: u16,
    segment_selector: u16,
    ty: EntryType,
    pub offset_mid: u16,
    pub offset_high: u32,
    _res: u32,
}

type Handler = unsafe extern "C" fn();

impl Entry {
    pub const fn empty() -> Entry {
        Entry {
            offset_low: 0,
            segment_selector: 0,
            ty: EntryType::new(),
            offset_mid: 0,
            offset_high: 0,
            _res: 0,
        }
    }

    fn set_handler(&mut self, func: Handler) {
        let addr = func as u64;
        self.offset_low = addr as u16;
        self.offset_mid = (addr >> 16) as u16;
        self.offset_high = (addr >> 32) as u32;
        self.segment_selector = 0x18;

        self.ty.set_present(true);
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
