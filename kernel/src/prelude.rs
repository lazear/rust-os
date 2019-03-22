#![allow(dead_code)]

use crate::sync::Global;
use crate::term::Terminal;
use core::fmt;
use core::mem;
use core::ops::Range;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::prelude::print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::prelude::println(format_args!($($arg)*));
    };
}

pub fn print(args: fmt::Arguments) {
    use fmt::Write;
    {
        let mut term = Terminal::global().lock();
        let _ = term.write_fmt(args);
    }
}

pub fn println(args: fmt::Arguments) {
    use fmt::Write;
    {
        let mut term = Terminal::global().lock();
        let _ = term.write_fmt(args);
        term.write_char('\n');
    }
}

pub trait BitField {
    fn bits(&self) -> u8;
    fn get_bit(&self, bit: u8) -> bool;
    fn get_bits(&self, bits: Range<u8>) -> Self;
    fn set_bit(&mut self, bit: u8, value: bool);
    fn set_bits(&mut self, bits: Range<u8>, value: Self);

    /// Return an iterator over the bits in
    fn bit_iter(&self) -> BitfieldIterator<Self>
    where
        Self: Copy,
    {
        BitfieldIterator {
            inner: *self,
            fwd_bit: 0,
            rev_bit: Some(self.bits() - 1),
        }
    }
}

pub struct BitfieldIterator<T: BitField> {
    fwd_bit: u8,
    rev_bit: Option<u8>,
    inner: T,
}

impl<T: BitField> Iterator for BitfieldIterator<T> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.fwd_bit < self.inner.bits() {
            let val = self.inner.get_bit(self.fwd_bit);
            self.fwd_bit += 1;
            Some(val)
        } else {
            None
        }
    }
}

impl<T: BitField> DoubleEndedIterator for BitfieldIterator<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.rev_bit.take() {
            Some(n) => {
                let val = self.inner.get_bit(n);
                self.rev_bit = if n == 1 { None } else { Some(n - 1) };
                Some(val)
            }
            None => None,
        }
    }
}

#[macro_export]
macro_rules! bitfield {
    ($($t:ty)*) => {$(
        impl BitField for $t {

            #[inline]
            fn bits(&self) -> u8 {
                mem::size_of::<Self>() as u8 * 8
            }
            fn get_bit(&self, bit: u8) -> bool {
                assert!(bit < self.bits());
                (*self & (1 << bit)) != 0
            }

            fn get_bits(&self, bits: Range<u8>) -> Self {
                let mut ret: Self = 0;
                for bit in bits.start .. bits.end {
                    ret.set_bit(bit, self.get_bit(bit));
                }
                ret
            }

            fn set_bit(&mut self, bit: u8, value: bool) {
                assert!(bit < self.bits());
                if value {
                    *self |= 1 << bit;
                } else {
                    *self &= !(1 << bit);
                }
            }

            fn set_bits(&mut self, bits: Range<u8>, value: Self) {
                for (bit, mask) in (bits.start .. bits.end).zip(value.bit_iter()) {
                    self.set_bit(bit, mask);
                }
            }
        }
    )*};
}

bitfield!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);

#[cfg(test)]
mod bitfield_test {
    use super::*;

    #[test]
    fn flip() {
        let mut f = 0b1110_0101u8;
        assert_eq!(f.bits(), 8);
        assert_eq!(f.get_bit(0), true);
        assert_eq!(f.get_bit(1), false);
        assert_eq!(f.get_bit(7), true);
        f.set_bit(7, false);
        assert_eq!(f, 0b0110_0101u8);
        assert_eq!(f.get_bit(7), false);
    }

    #[test]
    fn set_bits() {
        let mut f = 0u16;
        f.set_bits(8..12, 0b1010u16);
        assert_eq!(f, 0b0000_1010_0000_0000);
        f.set_bits(8..12, 0b0101u16);
        assert_eq!(f, 0b0000_0101_0000_0000);
    }

    #[test]
    #[should_panic]
    fn out_of_bounds() {
        let mut f = 0xFFu8;
        assert_eq!(f.get_bit(24), false);
        f.set_bit(24, true);
        assert_eq!(f.get_bit(24), true);
    }

    #[test]
    fn bit_range() {
        let f = 0b0110_0111_0010_0000u16;
        assert_eq!(f.get_bits(5..10), 0b0011_0010_0000u16);
    }

    #[test]
    fn bit_iter() {
        let f = 0b1110_0101u8;
        let r = vec![true, false, true, false, false, true, true, true];
        assert_eq!(f.bit_iter().collect::<Vec<bool>>(), r);
    }
}
