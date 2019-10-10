//! VGA terminal writing utilities
use super::io::{Io, Port, Volatile};
use super::sync;

global!(Terminal);

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// VGA color attributes
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
/// Combined foreground and background [`Color`] for printing to VGA terminal
///
/// Not visible to rest of crate
struct TextColor(u8);

impl TextColor {
    pub fn new(fg: Color, bg: Color) -> TextColor {
        TextColor((fg as u8) | ((bg as u8) << 4))
    }
}

impl Default for TextColor {
    fn default() -> TextColor {
        TextColor::new(Color::White, Color::Black)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
/// Struct representing a VGA terminal character/color pair
pub struct Character {
    byte: u8,
    color: TextColor,
}

impl Character {
    fn new(byte: u8, color: TextColor) -> Character {
        Character { byte, color }
    }
}

/// Handles writing to VGA video memory
pub struct Terminal {
    buffer: &'static mut [[Volatile<Character>; 80]; 25],
    pos: usize,
    color: TextColor,
}

impl Default for Terminal {
    /// Create a new [`Terminal`] struct that writes to VGA video memory
    ///
    /// It's not necessarily *unsafe* to call this twice, but it may
    /// have unintented consequences - i.e. causing a data race to write
    /// to the same areas of video memory. Therefore, a singular [`Terminal`]
    /// should be created and stored behind a [`Mutex`]
    fn default() -> Terminal {
        Terminal {
            buffer: unsafe { &mut *(0xB8000 as *mut _) },
            pos: 0,
            color: TextColor::default(),
        }
    }
}

impl core::fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.bytes().for_each(|b| self.write_byte(b));
        Ok(())
    }
}

impl Terminal {
    /// Update VGA cursor position
    fn move_cursor(&self) {
        let mut d4: Port<u16> = Port::new(0x03D4);
        let mut d5: Port<u16> = Port::new(0x03D5);
        let pos = (24 * 80) + (self.pos as u16);
        d4.write(14);
        d5.write(pos >> 8);
        d4.write(15);
        d5.write(pos);
    }

    /// Write a `Character` to the VGA terminal
    fn write_character(&mut self, ch: Character) {
        match ch.byte {
            b'\n' => return self.newline(),
            _ => {
                if self.pos == 79 {
                    self.newline();
                }
            }
        }
        self.buffer[24][self.pos].write(ch);
        self.pos += 1;
        self.move_cursor();
    }

    pub fn write_at(&mut self, text: &str, mut line: usize, mut pos: usize) {
        for &byte in text.as_bytes() {
            let ch = Character::new(byte, self.color);
            match ch.byte {
                b'\n' => return self.newline(),
                _ => {
                    if pos >= 79 {
                        line += 1;
                    }
                }
            }
            if line >= 24 {
                line = 24;
            }
            self.buffer[line][pos].write(ch);
            pos += 1;
            self.move_cursor();
        }
    }

    /// Move all lines on the screen up one row, removing the top row
    fn newline(&mut self) {
        for line in 1..25 {
            for col in 0..80 {
                let ch = self.buffer[line][col].read();
                self.buffer[line - 1][col].write(ch);
            }
        }
        for c in 0..self.pos {
            self.buffer[24][c].write(Character::new(0u8, self.color));
        }
        self.pos = 0;
        self.move_cursor();
    }

    /// Set the default foreground and background [`Color`] for the [`Terminal`]
    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color = TextColor::new(fg, bg);
    }

    /// Write a byte to VGA video memory, using the [`Terminal`]'s default color
    pub fn write_byte(&mut self, byte: u8) {
        self.write_character(Character::new(byte, self.color));
    }
}
