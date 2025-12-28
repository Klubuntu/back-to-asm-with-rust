use core::arch::{asm};


#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    White = 7,
    Gray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    LightWhite = 15,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }

    // Pomocnicza metoda, by łatwo wyciągnąć bajt u8
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

pub fn draw_block(col: u64, row: u64, color: u8) {
    // 0xDB to pełny blok (full block) w tablicy Code Page 437
    // Ustawiamy ten sam kolor dla znaku i tła, aby uzyskać jednolity prostokąt
    let full_color = (color << 4) | color; 
    vga_write!(col, row, 0xDBu8, full_color);
}