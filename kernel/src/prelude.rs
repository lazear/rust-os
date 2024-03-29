//! Provides support functions
#![allow(dead_code)]

pub use crate::sync::Global;
pub use core::fmt::Write;

use crate::term::Terminal;
use core::fmt;
use core::mem;
use core::ops::Range;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
            $crate::prelude::print(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
            $crate::print!("{}\n", format_args!($($arg)*));
    });
}

/// Print formatted [`fmt::Arguments`] to the global VGA terminal.
///
/// This function locks the global terminal
pub fn print(args: fmt::Arguments) {
    use fmt::Write;
    Terminal::global().lock().write_fmt(args).unwrap();
}

pub struct BytesBuf<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> BytesBuf<'a> {
    pub fn from_slice(slice: &'a mut [u8]) -> BytesBuf {
        BytesBuf {
            buf: slice,
            offset: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.buf) }
    }
}

impl<'a> core::fmt::Write for BytesBuf<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let src = s.as_bytes();
        let dst = &mut self.buf[self.offset..];
        if dst.len() < s.len() {
            return Err(core::fmt::Error);
        }
        let dst = &mut dst[..src.len()];
        dst.copy_from_slice(src);
        self.offset += src.len();
        Ok(())
    }
}

pub trait BitField {
    /// Return the number of bits in `self`
    fn bits(&self) -> u8;

    /// Get the boolean state of the given `bit` position
    fn get_bit(&self, bit: u8) -> bool;

    /// Return a bit mask covering `bits`
    fn get_bits(&self, bits: Range<u8>) -> Self;

    /// Set the provided `bit` to a boolean value
    fn set_bit(&mut self, bit: u8, value: bool);

    /// Fill in a range of `bits` from `value`. `bits` and `value` are
    /// zipped together, and the bits in `self` are filled in order
    /// starting from the 0th bit of `value`.
    fn set_bits(&mut self, bits: Range<u8>, value: Self);

    /// Return a [`BitIterator`] over the bits in `Self`
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

/// [`Iterator`] over the bits in a type that implements [`BitField`],
/// where forward iteration yields least-significant bits first and
/// reverse iteration (through [`DoubleEndedIterator`]) yields most-significant
/// bits first
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

/// Automatically implement [`BitField`] for numeric types
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

bitfield!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 usize isize);

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
