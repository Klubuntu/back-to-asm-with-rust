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

#[unsafe(no_mangle)]
pub fn unicode_menu() {
    // Czyścimy ekran standardowym makrem
    vga_clear!(0x00);

    // Ładujemy glify do karty graficznej
    unsafe { load_polish_fonts(); }

    // Wyświetlamy nagłówek
    vga_print!(0, 0, 0x0F, b"TRYB POLSKI (UTF8-MAPPED) AKTYWNY");

    // Zmapowany tekst: \x01=ą, \x02=ć, \x03=ę, \x04=ł, \x05=ń, \x06=ó, \x07=ś, \x08=ź, \x09=ż
    // Przykład: "Zażółć gęślą jaźń"
    let line1 = b"Zaza\x09\x06\x04\x02 g\x03\x07\x04\x01 ja\x08\x05";
    vga_print!(0, 2, 0x0E, line1);

    vga_print!(0, 4, 0x0B, b"Dostepne znaki:");
    vga_print!(0, 5, 0x0A, b"\x01 \x02 \x03 \x04 \x05 \x06 \x07 \x08 \x09");

    vga_print!(0, 24, 0x70, b" Nacisnij 0, aby wrocic do menu glownego ");
}

