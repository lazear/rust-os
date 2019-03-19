use super::io::{Io, Port, Volatile};
use core::ptr;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
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
struct TermColor(u8);

impl TermColor {
    pub fn new(fg: Color, bg: Color) -> TermColor {
        TermColor((fg as u8) | ((bg as u8) << 4))
    }
}

impl Default for TermColor {
    fn default() -> TermColor {
        TermColor::new(Color::White, Color::Black)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Character {
    ch: u8,
    color: TermColor,
}

impl Character {
    fn new(ch: char, color: TermColor) -> Character {
        Character {
            ch: ch as u8,
            color,
        }
    }
}

pub struct Writer {
    buffer: &'static mut [[Volatile<Character>; 80]; 25],
    pos: usize,
    color: TermColor,
}

impl Default for Writer {
    fn default() -> Writer {
        Writer {
            buffer: unsafe { &mut *(0xB8000 as *mut _) },
            pos: 0,
            color: TermColor::default()
        }
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.chars().for_each(|ch| self.write_char(ch));
        Ok(())
    }
}

impl Writer {

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
        match ch.ch as char {
            '\n' => return self.newline(),
            _ => {}
        }
        self.buffer[24][self.pos].write(ch);
        self.pos += 1;
        self.move_cursor();
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
            self.buffer[24][c].write(Character::new(' ', self.color));
        }
        self.pos = 0;
    }

    /// Set the default color for the terminal
    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color = TermColor::new(fg, bg);
    }

    pub fn write_char(&mut self, ch: char) {
        self.write_character(Character::new(ch, self.color));
    }
}