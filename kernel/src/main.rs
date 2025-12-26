#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::{asm,naked_asm};

// Minimalne shim-y
#[unsafe(no_mangle)]
pub extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        unsafe {
            let a = *s1.add(i);
            let b = *s2.add(i);
            if a != b { return (a as i32) - (b as i32); }
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn memcpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8 {
    for i in 0..n { unsafe { *d.add(i) = *s.add(i); } }
    d
}

#[unsafe(no_mangle)]
pub extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    for i in 0..n { unsafe { *s.add(i) = c as u8; } }
    s
}

// --- MAKRO ASM do VGA ---


macro_rules! vga_clear {
    ($color:expr) => {
        unsafe {
            let fill_value = (($color as u16) << 8) | (0x20u16); // 0x20 to spacja
            asm!(
                "cld",           // Wyczyść flagę kierunku (zwiększanie EDI)
                "rep stosw",     // Powtarzaj 'store word' (AX -> [EDI])
                inout("rcx") 80 * 25 => _, // Licznik powtórzeń (CX/RCX)
                inout("rdi") 0xb8000 => _, // Adres docelowy (DI/RDI)
                in("ax") fill_value,       // Wartość do wpisania
                options(nostack, preserves_flags)
            );
        }
    };
}

macro_rules! vga_clear_row {
    ($row:expr) => {
        unsafe {
            // Czyści pierwsze 20 kolumn w danym wierszu
            for i in 0..20 {
                write_char_macro!(i, $row, b' ', 0x00);
            }
        }
    };
}

// Pierwsza wersja makra do zapisu znaku na ekranie
macro_rules! vga_write {
    ($col:expr, $row:expr, $char:expr, $color:expr) => {
        unsafe {
            let offset = (($row * 80 + $col) * 2) as u64;
            let value = (($color as u16) << 8) | ($char as u16);
            
            asm!(
                // Używamy bezpośredniego adresu 0xb8000
                // {off:e} wymusza użycie 32-bitowej części rejestru (np. eax zamiast rax)
                "mov word ptr [0xb8000 + {off:e}], {val:x}",
                off = in(reg) offset,
                val = in(reg) value,
                options(nostack, preserves_flags)
            );
        }
    };
}

macro_rules! write_char_macro {
    ($col:expr, $row:expr, $ch:expr, $color:expr) => {
        unsafe {
            let off = (($row * 80 + $col) * 2) as u32;
            let val = (($color as u16) << 8) | ($ch as u16);
            asm!(
                "mov word ptr [0xb8000 + {off:e}], {val:x}",
                off = in(reg) off,
                val = in(reg) val,
                options(nostack) // Kluczowe: informujemy, że nie ruszamy stosu
            );
        }
    };
}

// MAKRO do obslugi klawatury
macro_rules! poll_keyboard {
    ($col:expr, $row:expr, $last_state:expr) => {
        unsafe {
            let scancode: u8;
            asm!("in al, 0x60", out("al") scancode);

            if scancode != $last_state {
                $last_state = scancode; // Aktualizujemy zmienną lokalną

                if scancode < 0x80 { // Reaguj tylko na wciśnięcie
                    // Czyścimy wiersz przed nowym napisem (np. pierwsze 10 znaków)
                    for i in 0..10 {
                        write_char_macro!(i, $row, b' ', 0x00);
                    }

                    match scancode {
                        0x02 => { // 1
                            write_char_macro!(0, $row, b'M', 0x0A);
                            write_char_macro!(1, $row, b'1', 0x0A);
                        }
                        0x03 => { // 2
                            write_char_macro!(0, $row, b'M', 0x0B);
                            write_char_macro!(1, $row, b'2', 0x0B);
                        }
                        0x04 => { // 3
                            write_char_macro!(0, $row, b'M', 0x0C);
                            write_char_macro!(1, $row, b'3', 0x0C);
                        }
                        _ => {}
                    }
                }
            }
        }
    };
}

macro_rules! poll_keyboard_simple {
    () => {
        unsafe {
            let scancode: u8;
            // 1. Pobierz kod
            asm!("in al, 0x60", out("al") scancode);

            // 2. Pobierz poprzedni stan z pamięci VGA (miejsce niewidoczne na ekranie)
            let last: u8;
            asm!("mov {0}, byte ptr [0xb8f9e]", out(reg_byte) last);

            // 3. Jeśli kod jest nowy i jest to "naciśnięcie"
            if scancode != last && scancode < 0x80 {
                // Zapisz stan
                asm!("mov byte ptr [0xb8f9e], {0}", in(reg_byte) scancode);

                // Klawisz '1' (0x02)
                if scancode == 0x02 {
                    vga_write!(0, 0, b'R', 0x0A);
                    vga_write!(1, 0, b'u', 0x0A);
                    vga_write!(2, 0, b's', 0x0A);
                    vga_write!(3, 0, b't', 0x0A);
                    vga_write!(4, 0, b'e', 0x0A);
                    vga_write!(5, 0, b'd', 0x0A);
                }
                
                // Klawisz '2' (0x03)
                if scancode == 0x03 {
                    vga_write!(0, 0, b'R', 0x0E);
                    vga_write!(1, 0, b'u', 0x0E);
                    vga_write!(2, 0, b's', 0x0E);
                    vga_write!(3, 0, b't', 0x0E);
                    vga_write!(4, 0, b'e', 0x0E);
                    vga_write!(5, 0, b'd', 0x0E);
                }

                // Klawisz '3' (0x04)
                if scancode == 0x04 {
                    vga_write!(0, 0, b'R', 0x0C);
                    vga_write!(1, 0, b'u', 0x0C);
                    vga_write!(2, 0, b's', 0x0C);
                    vga_write!(3, 0, b't', 0x0C);
                    vga_write!(4, 0, b'e', 0x0C);
                    vga_write!(5, 0, b'd', 0x0C);
                }
            }

            // 4. Jeśli puszczono klawisz, zresetuj schowek, by pozwolić na ponowne naciśnięcie
            if scancode >= 0x80 {
                asm!("mov byte ptr [0xb8f9e], 0");
            }
        }
    };
}

macro_rules! poll_keyboard_unified {
    ($row:expr) => {
        unsafe {
            let scancode: u8;
            asm!("in al, 0x60", out("al") scancode);

            // Odczyt poprzedniego stanu z pamięci VGA (bajt 3998)
            let last: u8;
            asm!("mov {0}, byte ptr [0xb8f9e]", out(reg_byte) last);

            if scancode != last {
                // Zapisujemy nowy stan do VGA
                asm!("mov byte ptr [0xb8f9e], {0}", in(reg_byte) scancode);

                if scancode < 0x80 { // Tylko naciśnięcia
                    // CZYŚCIMY wiersz (pierwsze 10 znaków)
                    // for i in 0..10 {
                    //     vga_write!(i, $row, b' ', 0x00);
                    // }

                    // Wyświetlamy napis w zależności od klawisza
                    if scancode == 0x02 { // Klawisz 1
                        let c = 0x0A; // Zielony
                        vga_write!(0, $row, b'R', c); 
                        vga_write!(1, $row, b'u', c);
                        vga_write!(2, $row, b's', c); 
                        vga_write!(3, $row, b't', c);
                        vga_write!(4, $row, b'e', c); 
                        vga_write!(5, $row, b'd', c);
                        write_char_macro!(0, $row, b'M', 0x0A);
                        write_char_macro!(1, $row, b'1', 0x0A);
                    } 
                    else if scancode == 0x03 { // Klawisz 2
                        let c = 0x0E; // Żółty
                        vga_write!(0, $row, b'R', c); 
                        vga_write!(1, $row, b'u', c);
                        vga_write!(2, $row, b's', c); 
                        vga_write!(3, $row, b't', c);
                        vga_write!(4, $row, b'e', c); 
                        vga_write!(5, $row, b'd', c);
                        write_char_macro!(0, $row, b'M', 0x0B);
                        write_char_macro!(1, $row, b'2', 0x0B);
                    }
                    else if scancode == 0x04 { // Klawisz 3
                        let c = 0x0C; // Czerwony
                        vga_write!(0, $row, b'R', c); 
                        vga_write!(1, $row, b'u', c);
                        vga_write!(2, $row, b's', c); 
                        vga_write!(3, $row, b't', c);
                        vga_write!(4, $row, b'e', c); 
                        vga_write!(5, $row, b'd', c);
                        write_char_macro!(0, $row, b'M', 0x0C);
                        write_char_macro!(1, $row, b'3', 0x0C);
                    }

                    else if scancode == 0x0B { // Klawisz 0
                        vga_write!(0, 0, b'R', 0x0F);
                        vga_write!(1, 0, b'u', 0x0A);
                        vga_write!(2, 0, b's', 0x0E);
                        vga_write!(3, 0, b't', 0x0C);
                        vga_write!(4, 0, b'e', 0x0B);
                        vga_write!(5, 0, b'd', 0x05);
                    }

                    else if scancode == 0x01 { // Klawisz Esc
                        vga_clear!(0x00); // Czarny ekran
                    }

                    else if scancode == 0x3F { // F5
                        unsafe {
                            // 1. Reset PCI (Najskuteczniejszy w QEMU)
                            asm!(
                                "out dx, al",
                                in("dx") 0xCF9u16,
                                in("al") 0x06u8,
                                options(nomem, nostack)
                            );

                            // 2. Reset przez kontroler klawiatury (to co masz w reboot_system)
                            asm!(
                                "out 0x64, al",
                                in("al") 0xFEu8,
                                options(nomem, nostack)
                            );

                            // 3. Triple Fault (Gwarantuje, że QEMU zareaguje)
                            asm!(
                                "push 0",
                                "push 0",
                                "lidt [rsp]",
                                "int 3",
                                options(noreturn, nostack)
                            );
                        }
                    }
                }
            }

            // Reset schowka przy puszczeniu klawisza (Break Code)
            if scancode >= 0x80 {
                asm!("mov byte ptr [0xb8f9e], 0");
            }
        }
    };
}

#[inline(always)] // Nadal bardzo szybkie
fn write_char(col: usize, row: usize, ch: u8, color: u8) {
    let off = ((row * 80 + col) * 2) as u32;
    let val = ((color as u16) << 8) | (ch as u16);
    unsafe {
        asm!(
            "mov word ptr [0xb8000 + {off:e}], {val:x}",
            off = in(reg) off,
            val = in(reg) val,
            options(nostack, preserves_flags)
        );
    }
}

#[unsafe(naked)]
#[unsafe(link_section = ".text._start")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        naked_asm!(
            "mov rsp, 0x90000",
            "sub rsp, 8",       // "Sztuczne" wyrównanie (padding)
            "jmp kernel_main", // Skaczemy do właściwego Rusta
        );
    }
}

// Ta funkcja jest bezpieczna, bo stos jest już ustawiony przez _start
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    vga_clear!(0x00); // Czarny ekran
    vga_write!(0, 0, b'R', 0x0F);
    vga_write!(1, 0, b'u', 0x0A);
    vga_write!(2, 0, b's', 0x0E);
    vga_write!(3, 0, b't', 0x0C);
    vga_write!(4, 0, b'e', 0x0B);
    vga_write!(5, 0, b'd', 0x05);

    // Click Key 1 - Mode1
    vga_write!(0, 2, b'C', 0x0F);
    vga_write!(1, 2, b'l', 0x0F);
    vga_write!(2, 2, b'i', 0x0F);
    vga_write!(3, 2, b'c', 0x0F);
    vga_write!(4, 2, b'k', 0x0F);
    vga_write!(6, 2, b'K', 0x0C);
    vga_write!(7, 2, b'e', 0x0C);
    vga_write!(8, 2, b'y', 0x0C);
    vga_write!(10, 2, b'1', 0x0C);
    vga_write!(12, 2, b'-', 0x0F);
    vga_write!(14, 2, b'M', 0x0A);
    vga_write!(15, 2, b'o', 0x0A);
    vga_write!(16, 2, b'd', 0x0A);
    vga_write!(17, 2, b'e', 0x0A);
    vga_write!(18, 2, b'1', 0x0A);

    // Click Key 2 - Mode2
    vga_write!(0, 3, b'C', 0x0F);
    vga_write!(1, 3, b'l', 0x0F);
    vga_write!(2, 3, b'i', 0x0F);
    vga_write!(3, 3, b'c', 0x0F);
    vga_write!(4, 3, b'k', 0x0F);
    vga_write!(6, 3, b'K', 0x0A);
    vga_write!(7, 3, b'e', 0x0A);
    vga_write!(8, 3, b'y', 0x0A);
    vga_write!(10, 3, b'2', 0x0A);
    vga_write!(12, 3, b'-', 0x0F);
    vga_write!(14, 3, b'M', 0x0B);
    vga_write!(15, 3, b'o', 0x0B);
    vga_write!(16, 3, b'd', 0x0B);
    vga_write!(17, 3, b'e', 0x0B);
    vga_write!(18, 3, b'2', 0x0B);

    // Click Key 3 - Mode3
    vga_write!(0, 4, b'C', 0x0F);
    vga_write!(1, 4, b'l', 0x0F);
    vga_write!(2, 4, b'i', 0x0F);
    vga_write!(3, 4, b'c', 0x0F);
    vga_write!(4, 4, b'k', 0x0F);
    vga_write!(6, 4, b'K', 0x0E);
    vga_write!(7, 4, b'e', 0x0E);
    vga_write!(8, 4, b'y', 0x0E);
    vga_write!(10, 4, b'3', 0x0E);
    vga_write!(12, 4, b'-', 0x0F);
    vga_write!(14, 4, b'M', 0x0C);
    vga_write!(15, 4, b'o', 0x0C);
    vga_write!(16, 4, b'd', 0x0C);
    vga_write!(17, 4, b'e', 0x0C);
    vga_write!(18, 4, b'3', 0x0C);

    // Click Key 0 - Start
    vga_write!(0, 5, b'C', 0x0F);
    vga_write!(1, 5, b'l', 0x0F);
    vga_write!(2, 5, b'i', 0x0F);
    vga_write!(3, 5, b'c', 0x0F);
    vga_write!(4, 5, b'k', 0x0F);
    vga_write!(6, 5, b'K', 0x0D);
    vga_write!(7, 5, b'e', 0x0D);
    vga_write!(8, 5, b'y', 0x0D);
    vga_write!(10, 5, b'0', 0x0D);
    vga_write!(12, 5, b'-', 0x0F);
    vga_write!(14, 5, b'S', 0x0E);
    vga_write!(15, 5, b't', 0x0E);
    vga_write!(16, 5, b'a', 0x0E);
    vga_write!(17, 5, b'r', 0x0E);
    vga_write!(18, 5, b't', 0x0E);

    // Click ESC - Reset
    vga_write!(0, 7, b'C', 0x0F);
    vga_write!(1, 7, b'l', 0x0F);
    vga_write!(2, 7, b'i', 0x0F);
    vga_write!(3, 7, b'c', 0x0F);
    vga_write!(4, 7, b'k', 0x0F);
    vga_write!(6, 7, b'E', 0x05);
    vga_write!(7, 7, b'S', 0x05);
    vga_write!(8, 7, b'C', 0x05);
    vga_write!(10, 7, b'-', 0x0F);
    vga_write!(12, 7, b'R', 0x04);
    vga_write!(13, 7, b'E', 0x04);
    vga_write!(14, 7, b'S', 0x04);
    vga_write!(15, 7, b'E', 0x04);
    vga_write!(16, 7, b'T', 0x04);


    // Click F5 - Reboot
    vga_write!(0, 8, b'C', 0x0F);
    vga_write!(1, 8, b'l', 0x0F);
    vga_write!(2, 8, b'i', 0x0F);
    vga_write!(3, 8, b'c', 0x0F);
    vga_write!(4, 8, b'k', 0x0F);
    vga_write!(6, 8, b'F', 0x02);
    vga_write!(7, 8, b'5', 0x02);;
    vga_write!(10, 8, b'-', 0x0F);
    vga_write!(12, 8, b'R', 0x06);
    vga_write!(13, 8, b'E', 0x06);
    vga_write!(14, 8, b'B', 0x06);
    vga_write!(15, 8, b'O', 0x06);
    vga_write!(16, 8, b'O', 0x06);
    vga_write!(17, 8, b'T', 0x06);

    unsafe { asm!("mov byte ptr [0xb8f9e], 0"); }

    loop { 
        unsafe { 
            let status: u8;
            // Odczyt portu statusu klawiatury (0x64)
            asm!("in al, 0x64", out("al") status);
            
            // Bit 0 informuje, czy w porcie 0x60 są dane do odczytu
            if status & 0x01 != 0 {
                poll_keyboard_unified!(0); // Obsługa klawiatury dla wiersza 0
            }
            
            asm!("pause");
         } 
    }
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    vga_write!(0, 0, b'E', 0x4F);
    vga_write!(1, 0, b'R', 0x4F);
    vga_write!(2, 0, b'2', 0x4F);
    vga_write!(3, 0, b'5', 0x4F);
    loop { unsafe { asm!("hlt"); } }
}