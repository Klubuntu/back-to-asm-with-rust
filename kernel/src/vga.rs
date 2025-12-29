use core::arch::{asm};
// use crate::{vga_clear, vga_print, vga_write};


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

#[allow(dead_code)]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorCode(u8);

#[allow(dead_code)]
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

pub fn write_char(col: u64, row: u64, ch: u8, color: u8) {
    let offset = ((row * 80 + col) * 2) as u64;
    let value = ((color as u16) << 8) | (ch as u16);
    unsafe {
        asm!(
            "mov word ptr [0xb8000 + {off:e}], {val:x}",
            off = in(reg) offset,
            val = in(reg) value,
            options(nostack, preserves_flags)
        );
    }
}

pub fn clear(color: u8) {
    let fill_value = ((color as u16) << 8) | 0x20u16;
    unsafe {
        asm!(
            "cld",
            "rep stosw",
            inout("rcx") 80 * 25 => _,
            inout("rdi") 0xb8000 => _,
            in("ax") fill_value,
            options(nostack, preserves_flags)
        );
    }
}

pub fn print_bytes(col: u64, row: u64, color: u8, bytes: &[u8]) {
    let mut c = col;
    for &b in bytes {
        write_char(c, row, b, color);
        c += 1;
    }
}

// VGA MODE
// Bitmapy polskich znaków (format 8x16)
const GLYPH_A_OGONEK: [u8; 16] = [0,0,0,124,1,127,129,129,127,1,6,4,0,0,0,0];
const GLYPH_C_ACUTE:  [u8; 16] = [0,8,4,124,128,128,128,128,124,0,0,0,0,0,0,0];
const GLYPH_E_OGONEK: [u8; 16] = [0,0,0,124,130,254,128,126,130,124,4,8,0,0,0,0];
const GLYPH_L_STROKE: [u8; 16] = [0,128,128,136,144,160,128,128,128,0,0,0,0,0,0,0];
const GLYPH_N_ACUTE:  [u8; 16] = [0,8,4,130,194,162,146,138,134,130,0,0,0,0,0,0];
const GLYPH_O_ACUTE:  [u8; 16] = [0,8,4,124,130,130,130,130,124,0,0,0,0,0,0,0];
const GLYPH_S_ACUTE:  [u8; 16] = [0,8,4,124,128,124,2,124,0,0,0,0,0,0,0,0];
const GLYPH_Z_ACUTE:  [u8; 16] = [0,8,4,254,4,8,16,32,254,0,0,0,0,0,0,0];
const GLYPH_Z_DOT:    [u8; 16] = [0,16,0,254,4,8,16,32,254,0,0,0,0,0,0,0];

unsafe fn load_polish_fonts() {
    // 1. Przygotowanie sekwencera VGA do zapisu w Plane 2 (pamięć fontów)
    // Wykorzystujemy instrukcje 'out' do komunikacji z portami I/O VGA
    unsafe {
        asm!(
            "mov dx, 0x3C4", "mov ax, 0x0402", "out dx, ax", // Wybierz Plane 2
            "mov dx, 0x3CE", "mov ax, 0x0005", "out dx, ax", // Wyłącz tryb parzysty/nieparzysty
            "mov ax, 0x0406", "out dx, ax",                 // Mapuj Plane 2 na A0000
            "mov dx, 0x3C4", "mov ax, 0x0704", "out dx, ax", // Dostęp sekwencyjny
        );
    }

    let vga_font_mem = 0xA0000 as *mut u8;
    let glyphs = [
        &GLYPH_A_OGONEK, &GLYPH_C_ACUTE, &GLYPH_E_OGONEK, &GLYPH_L_STROKE,
        &GLYPH_N_ACUTE, &GLYPH_O_ACUTE, &GLYPH_S_ACUTE, &GLYPH_Z_ACUTE, &GLYPH_Z_DOT
    ];

    // Nadpisujemy pierwsze 9 znaków ASCII (indeksy 1-9)
    // Każdy znak w pamięci zajmuje dokładnie 32 bajty (używamy 16)
    for (idx, glyph) in glyphs.iter().enumerate() {
        for i in 0..16 {
            unsafe {
                *vga_font_mem.add((idx + 1) * 32 + i) = (*glyph)[i];
            }
        }
    }

    // 2. Przywrócenie standardowego trybu tekstowego, aby widzieć tekst
    unsafe {
        asm!(
            "mov dx, 0x3C4", "mov ax, 0x0302", "out dx, ax",
            "mov ax, 0x0304", "out dx, ax",
            "mov dx, 0x3CE", "mov ax, 0x1005", "out dx, ax",
            "mov ax, 0x0E06", "out dx, ax",
        );
    }
}

#[cfg(feature = "unicode")]
#[unsafe(no_mangle)]
pub fn unicode_menu() {
    // Czyścimy ekran
    vga_clear!(0x00);

    // Nagłówek i instrukcje w czystym ASCII
    vga_print!(0, 0, 0x0F, b"UNICODE MENU (ASCII)");
    vga_print!(0, 2, 0x0E, b"Polskie znaki w ASCII nie beda widoczne");
    vga_print!(0, 4, 0x0A, b"Key 0 - Back to Main Menu");
    vga_print!(0, 6, 0x09, b"Key 1 - (brak 720p w kernelu)");
    vga_print!(0, 24, 0x70, b" Nacisnij 0, aby wrocic do menu glownego ");
}

#[cfg(not(feature = "unicode"))]
#[unsafe(no_mangle)]
pub fn unicode_menu() {
    vga_clear!(0x00);
    vga_print!(0, 0, 0x0E, b"Unicode menu disabled in this build");
    vga_print!(0, 2, 0x07, b"Enable feature 'unicode' to restore it.");
}

