#![allow(unused_unsafe)]

use core::arch::asm;

// Utils macros
macro_rules! sleep_time { // in ms
    ($seconds:expr) => {
        // Przyjmujemy, że ok. 2 000 000 000 pętli to mniej więcej 1 sekunda 
        // (wartość zależy od taktowania, w QEMU może być różnie)
        let iterations = $seconds as u64 * 100_000; 
        unsafe {
            asm!(
                "2:",
                "nop",
                "dec {count}",
                "jnz 2b",
                count = inout(reg) iterations => _,
                options(nostack, preserves_flags)
            );
        }
    };
}

#[macro_export]
macro_rules! get_random {
    () => {
        $crate::kernel_main::next_rand()
    };
}

#[macro_export]
macro_rules! set_encoding {
    (UTF8) => {
        unsafe { asm!("mov byte ptr [0x508], 1"); } // Flaga trybu pod nowym adresem
    };
    (ASCII) => {
        unsafe { asm!("mov byte ptr [0x508], 0"); }
    };
}


// VGA Macros

#[macro_export]
macro_rules! vga_clear {
    ($color:expr) => {
        $crate::vga::clear($color as u8);
    };
}

#[macro_export]
macro_rules! vga_clear_animated {
    // Wersja z określoną liczbą znaków
    ($col:expr, $row:expr, $color:expr, $ms:expr, $num_chars:expr) => {
        unsafe {
            let mut current_col = $col as u64;
            let mut current_row = $row as u64;
            let num = $num_chars as u64;
            
            // Obliczamy opóźnienie na jeden znak
            let delay_per_char = ($ms as u64 * 1_000_000) / num;

            for _ in 0..num {
                // Jeśli wyjdziemy poza szerokość ekranu, przechodzimy do nowej linii
                if current_col >= 80 {
                    current_col = 0;
                    current_row += 1;
                }
                
                // Czyścimy znak (0x20 to spacja)
                vga_write!(current_col, current_row, 0x20u8, $color);

                // Pętla opóźniająca
                let c = delay_per_char;
                if c > 0 {
                    asm!(
                        "2:",
                        "nop",
                        "dec {count}",
                        "jnz 2b",
                        count = in(reg) c,
                        options(nostack, preserves_flags)
                    );
                }
                current_col += 1;
            }
        }
    };

    // Wersja domyślna (cały rząd od podanej kolumny do końca rzędu)
    ($col:expr, $row:expr, $color:expr, $ms:expr) => {
        vga_clear_animated!($col, $row, $color, $ms, 80 - $col);
    };
}

#[macro_export]
macro_rules! vga_write {
    ($col:expr, $row:expr, $char:expr, $color:expr) => {
        $crate::vga::write_char($col as u64, $row as u64, $char as u8, $color as u8);
    };
}

#[macro_export]
// print via LODSB instruction
macro_rules! vga_print {
    ($col:expr, $row:expr, $color:expr, $text:expr) => {
        $crate::vga::print_bytes($col as u64, $row as u64, $color as u8, $text);
    };
}

#[macro_export]
macro_rules! vga_print_ext {
    ($col:expr, $row:expr, $text:expr, $colors:expr) => {
        unsafe {
            let vga_start = 0xb8000 + (($row * 80 + $col) * 2) as u64;
            let text_ptr = $text.as_ptr();
            let color_ptr = $colors.as_ptr();
            let len = $text.len();

            asm!(
                "test {len}, {len}",
                "jz 3f",
                "2:",
                "mov al, [rsi]",      // Pobierz znak
                "mov ah, [rdx]",      // Pobierz kolor
                "mov [rdi], ax",      // Zapisz do VGA
                "add rsi, 1",
                "add rdx, 1",
                "add rdi, 2",
                "dec {len}",
                "jnz 2b",
                "3:",
                len = inout(reg) len => _,
                inout("rsi") text_ptr => _,   
                inout("rdx") color_ptr => _,  
                inout("rdi") vga_start => _,
                out("ax") _,
                options(nostack)
            );
        }
    };
}

#[macro_export]
macro_rules! vga_input_setup {
    () => {
        // Rysujemy pasek inputu na dole (linia 24)
        vga_draw_rect!(0, 24, 80, 1, 0x07); // Szary pasek
        vga_print!(0, 24, 0x70, b" INPUT > "); // Czarny tekst na szarym
    };
}

#[macro_export]
macro_rules! write_char_macro {
    ($col:expr, $row:expr, $ch:expr, $color:expr) => {
        // Redirect to safe runtime function which performs bounds checks.
        $crate::vga::write_char($col as u64, $row as u64, $ch as u8, $color as u8);
    };
}

#[macro_export]
macro_rules! vga_draw_rect {
    ($col:expr, $row:expr, $w:expr, $h:expr, $color:expr) => {
        unsafe {
            let mut r = 0;
            while r < $h {
                let mut c = 0;
                while c < $w {
                    $crate::vga::draw_block($col + c, $row + r, $color);
                    c += 1;
                }
                r += 1;
            }
        }
    };
}

// Unicode and Graphics Macros
#[macro_export]
macro_rules! draw_unicode_char {
    ($x:expr, $y:expr, $text_ptr:expr) => {
        unsafe {
            asm!(
                "movzx eax, byte ptr [rsi]",
                "cmp al, 0x80",
                "jb .is_ascii",
                
                // Logika dla 2 bajtów (np. ą, ć, ę...)
                "movzx ebx, byte ptr [rsi + 1]",
                "and al, 0x1F", // Maskowanie nagłówka 110xxxxx
                "shl al, 6",
                "and bl, 0x3F", // Maskowanie kontynuacji 10xxxxxx
                "or al, bl",
                "jmp .draw",

                ".is_ascii:",
                // Standardowe ASCII
                
                ".draw:",
                // Tutaj EAX zawiera numer znaku Unicode.
                // Teraz musisz go "narysować" pikselami, a nie wysłać do 0xb8000.
                in("rsi") $text_ptr,
                // ... reszta rejestrów
            );
        }
    };
}

#[macro_export]
macro_rules! poll_keyboard_unified {
    ($row:expr) => {
        let scancode: u8;
        unsafe { asm!("in al, 0x60", out("al") scancode); }

        let last: u8;
        unsafe { asm!("mov {0}, byte ptr [0x501]", out(reg_byte) last); }

        let menu_state: u8;
        unsafe { asm!("mov {0}, byte ptr [0x500]", out(reg_byte) menu_state); }

        let input_mode: u8;
        unsafe { asm!("mov {0}, byte ptr [0x506]", out(reg_byte) input_mode); }

        // Odczyt flagi ALT (używamy adresu 0x507)
        let mut alt_pressed: u8;
        unsafe { asm!("mov {0}, byte ptr [0x507]", out(reg_byte) alt_pressed); }

        if scancode != last {
            unsafe { asm!("mov byte ptr [0x501], {0}", in(reg_byte) scancode); }

            // Debug: pokaż ostatni scancode (również >= 0x80) w prawym górnym rogu
            let hi = (scancode >> 4) & 0x0F;
            let lo = scancode & 0x0F;
            let mut hi_ch = hi + b'0';
            if hi >= 10 { hi_ch = (hi - 10) + b'A'; }
            let mut lo_ch = lo + b'0';
            if lo >= 10 { lo_ch = (lo - 10) + b'A'; }
            vga_write!(74, 0, b'S', 0x0F);
            vga_write!(75, 0, b'C', 0x0F);
            vga_write!(77, 0, hi_ch, 0x0E);
            vga_write!(78, 0, lo_ch, 0x0E);

            // --- OBSŁUGA ALT (Make & Break) ---
            if scancode == 0x38 { // ALT Pressed
                unsafe { asm!("mov byte ptr [0x507], 1"); }
                alt_pressed = 1;
            } else if scancode == 0xB8 { // ALT Released
                unsafe { asm!("mov byte ptr [0x507], 0"); }
                alt_pressed = 0;
            }

            if scancode < 0x80 { // Make code
                if input_mode == 1 {
                    // --- LOGIKA TRYBU INPUT ---
                    if scancode == 0x1C { // ENTER
                        let len: u8;
                        unsafe { asm!("mov {0}, byte ptr [0x505]", out(reg_byte) len); }
                        let input_ctx: u8;
                        unsafe { asm!("mov {0}, byte ptr [0x511]", out(reg_byte) input_ctx); }

                        if len > 0 {
                            if input_ctx == 1 {
                                // Tworzenie pliku z nazwy wpisanej w buforze 0x600
                                let mut name_buf = [b' '; 8];
                                let count: u8 = if len > 8 { 8 } else { len };
                                let mut idx: u8 = 0;
                                while idx < count {
                                    let c: u8;
                                    let off = 0x600 + idx as u64;
                                    unsafe { asm!("mov {0}, byte ptr [{1}]", out(reg_byte) c, in(reg) off); }
                                    let uc = if c >= b'a' && c <= b'z' { c - 32 } else { c };
                                    name_buf[idx as usize] = uc;
                                    idx += 1;
                                }
                                unsafe {
                                    $crate::fs::fat16::fat16_create_file(&name_buf);
                                    asm!("mov byte ptr [0x506], 0"); // wyjscie z input
                                    asm!("mov byte ptr [0x505], 0"); // reset dlugosci
                                    asm!("mov byte ptr [0x511], 0"); // reset kontekstu
                                }
                                vga_print!(0, 22, 0x0A, b"Plik utworzony");
                                sleep_time!(250);
                                vga_clear!(0x00);
                                $crate::fs::fat16::fat16_mc();
                            } else {
                                // Domyślna prezentacja ostatniego inputu
                                vga_print!(0, 22, 0x0E, b"OSTATNI INPUT: \0");
                                for i in 0..len {
                                    let character: u8;
                                    let offset = 0x600 + i as u64;
                                    unsafe { asm!("mov {0}, byte ptr [{1}]", out(reg_byte) character, in(reg) offset); }
                                    vga_write!(15 + i as u64, 22, character, 0x0F);
                                }
                                unsafe { 
                                    asm!("mov byte ptr [0x506], 0"); 
                                    asm!("mov byte ptr [0x505], 0"); 
                                }
                                sleep_time!(500);
                                vga_clear_animated!(0, 24, 0x00, 300, 100);
                            }
                        }
                    }
                    else if scancode == 0x01 { // ESC
                        unsafe { asm!("mov byte ptr [0x506], 0"); }
                        vga_clear_animated!(0, 24, 0x00, 300, 100);
                    }
                    else if scancode == 0x0E { // BACKSPACE
                        let mut len: u8;
                        unsafe { asm!("mov {0}, byte ptr [0x505]", out(reg_byte) len); }
                        if len > 0 {
                            len -= 1;
                            unsafe { asm!("mov byte ptr [0x505], {0}", in(reg_byte) len); }
                            vga_write!(9 + len as u64, 24, b' ', 0x70);
                        }
                    }
                    else if scancode == 0x47 { // HOME -> Mini-Commander z trybu input
                        unsafe {
                            asm!("mov byte ptr [0x510], 0"); // reset wyboru
                            asm!("mov byte ptr [0x506], 0"); // wyjscie z input mode
                            asm!("mov byte ptr [0x500], 3"); // stan: Mini-Commander
                        }
                        vga_clear!(0x00);
                        $crate::fs::fat16::fat16_mc();
                    }
                    else if scancode == 0x47 { // HOME BTN
                        unsafe { 
                            asm!("mov byte ptr [0x510], 0"); // Resetujemy wybór na pierwszy element (indeks 0)
                        }
                        $crate::fs::fat16::fat16_mc();
                    }
                    else if scancode != 0x38 { // Mapowanie ASCII + POLSKIE
                        let ascii = if alt_pressed == 1 {
                            match scancode {
                                0x1E => 0x01, // Alt+A -> ą
                                0x2E => 0x02, // Alt+C -> ć
                                0x12 => 0x03, // Alt+E -> ę
                                0x26 => 0x04, // Alt+L -> ł
                                0x31 => 0x05, // Alt+N -> ń
                                0x18 => 0x06, // Alt+O -> ó
                                0x1F => 0x07, // Alt+S -> ś
                                0x2C => 0x08, // Alt+X -> ź
                                0x2D => 0x09, // Alt+Z -> ż
                                _ => 0,
                            }
                        } else {
                            match scancode {
                                0x1E => b'A', 0x30 => b'B', 0x2E => b'C', 0x20 => b'D', 0x12 => b'E',
                                0x21 => b'F', 0x22 => b'G', 0x23 => b'H', 0x17 => b'I', 0x24 => b'J',
                                0x25 => b'K', 0x26 => b'L', 0x32 => b'M', 0x31 => b'N', 0x18 => b'O',
                                0x19 => b'P', 0x10 => b'Q', 0x13 => b'R', 0x1F => b'S', 0x14 => b'T',
                                0x16 => b'U', 0x2F => b'V', 0x11 => b'W', 0x2D => b'X', 0x15 => b'Y',
                                0x2C => b'Z', 0x39 => b' ', 
                                0x02 => b'1', 0x03 => b'2', 0x04 => b'3', 0x05 => b'4', 0x06 => b'5',
                                0x07 => b'6', 0x08 => b'7', 0x09 => b'8', 0x0A => b'9', 0x0B => b'0',
                                0x0C => b'-', 0x0D => b'=', 0x1A => b'[', 0x1B => b']', 0x27 => b';',
                                0x28 => b'\'', 0x29 => b'`', 0x2B => b'\\', 0x33 => b',', 0x34 => b'.',
                                0x35 => b'/',
                                _ => 0,
                            }
                        };
                        
                        if ascii != 0 {
                            let mut len: u8;
                            unsafe { asm!("mov {0}, byte ptr [0x505]", out(reg_byte) len); }
                            if len < 60 {
                                let offset = 0x600 + len as u64;
                                unsafe { asm!("mov byte ptr [{addr}], {val}", addr = in(reg) offset, val = in(reg_byte) ascii); }
                                vga_write!(9 + len as u64, 24, ascii, 0x70);
                                unsafe { asm!("inc byte ptr [0x505]"); }
                            }
                        }
                    }
                } else {
                    // --- LOGIKA MENU ---
                    if menu_state == 0 { // MAIN MENU
                        if scancode == 0x02 { vga_print!(0, $row, 0x0A, b"Rusted M1"); } 
                        else if scancode == 0x03 { vga_print!(0, $row, 0x0E, b"Rusted M2"); }
                        else if scancode == 0x04 { vga_print!(0, $row, 0x0C, b"Rusted M3"); }
                        else if scancode == 0x05 { // Key 4 - Line (rysuj poziomą linię)
                            vga_draw_rect!(0, 12, 80, 1, 0x02);
                        }
                        else if scancode == 0x0A { /* Logika Secrets... */ }
                        else if scancode == 0x32 { // M - Math
                            unsafe { asm!("mov byte ptr [0x500], 1"); }
                            vga_clear!(0x00);
                            vga_print!(0, 0, 0x0F, b"MATH MENU: 1-Add, 2-Sub, 3-Mul, 4-Div, 9-Rand, 0-Back");
                        }
                        else if scancode == 0x17 { // I - Input
                            unsafe { asm!("mov byte ptr [0x506], 1"); }
                            vga_input_setup!();
                        }
                        else if scancode == 0x47 { // HOME - Mini-Commander
                            unsafe {
                                asm!("mov byte ptr [0x510], 0"); // zaznaczenie na pierwszy element
                                asm!("mov byte ptr [0x500], 3");  // stan: Mini-Commander
                            }
                            $crate::fs::fat16::fat16_mc();
                        }
                        else if scancode == 0x42 || scancode == 0x07 || scancode == 0x64 || scancode == 0x0A || scancode == 0x09 || scancode == 0xF0 { // F8 - Unicode (handle set1/2/3 and 0xF0 prefix)
                            unsafe { 
                                asm!("mov byte ptr [0x508], 1");
                                asm!("mov byte ptr [0x500], 2"); 
                            }
                            unicode_menu();
                        }
                        else if scancode == 0x01 { vga_clear!(0x00); }
                    } else if menu_state == 1 {
                        // --- LOGIKA MATH MENU ---
                        if scancode == 0x0B { // Klawisz 0 - Powrót
                            main_menu();
                        }
                        else if scancode == 0x02 { // Klawisz 1 w Math
                            vga_print!(0, 10, 0x0A, b"Addition selected!");
                            let add_result = 2 + 2;
                            vga_print!(0, 11, 0x0F, b"2 + 2 = ");
                            let res_char = (add_result as u8) + b'0';
                            vga_write!(8, 11, res_char, 0x0F);
                        } 
                        else if scancode == 0x03 { // Klawisz 2 w Math
                            vga_print!(0, 10, 0x0E, b"Subtraction selected!");
                            let sub_result = 5 - 3;
                            vga_print!(0, 11, 0x0F, b"5 - 3 = ");
                            let res_char = (sub_result as u8) + b'0';
                            vga_write!(8, 11, res_char, 0x0F);
                        }
                        else if scancode == 0x04 { // Klawisz 3 w Math
                            vga_print!(0, 10, 0x0C, b"Multiplication selected!");
                            let mul_result = 3 * 4;
                            vga_print!(0, 11, 0x0F, b"3 * 4 = ");
                            let res_char = (mul_result as u8) + b'0';
                            vga_write!(8, 11, res_char, 0x0F);
                        }
                        else if scancode == 0x05 { // Klawisz 4 w Math
                            vga_print!(0, 10, 0x09, b"Division selected!");
                            let div_result = 8 / 2;
                            vga_print!(0, 11, 0x0F, b"8 / 2 = ");
                            let res_char = (div_result as u8) + b'0';
                            vga_write!(8, 11, res_char, 0x0F);
                        }
                        else if scancode == 0x0A { // Klawisz 9 w Math
                            let random_number = (get_random!() % 100) as u8; // Zakres 0-99
                            let tens = random_number / 10;
                            let ones = random_number % 10;

                            vga_print!(0, 10, 0x07, b"Random Number: ");
                            let mut col = 15;
                            if tens > 0 {
                                vga_write!(col, 10, tens + b'0', 0x07);
                                col += 1;
                            }
                            vga_write!(col, 10, ones + b'0', 0x07);
                        }
                        // Tutaj możesz dodać resztę klawiszy dla Math
                    } else if menu_state == 2 {
                    // --- LOGIKA UNICODE MENU ---
                        if scancode == 0x0B { // Klawisz 0 - Powrót do Main Menu
                            main_menu();
                        }
                        else if scancode == 0x02 { // Klawisz 1 - 720p Demo
                            vga_clear!(0x00);
                            vga_print!(0, 5, 0x0C, b"Tryb 720p wymaga VM86!");
                            vga_print!(0, 6, 0x0C, b"W kernelu nie mozna uzywac INT 0x10");
                            vga_print!(0, 8, 0x0E, b"Nacisnij 0 aby wrocic");
                        }
                    
                    } else if menu_state == 3 { // Tryb Mini-Commandera
                        if scancode == 0x48 { // STRZALKA W GORE
                            // Dekrementuj tylko gdy zaznaczenie > 0
                            let sel: u8;
                            unsafe { asm!("mov {0}, byte ptr [0x510]", out(reg_byte) sel); }
                            if sel > 0 { unsafe { asm!("dec byte ptr [0x510]"); } }
                            $crate::fs::fat16::refresh_mc_list();
                        }
                        else if scancode == 0x50 { // STRZALKA W DOL
                            // Inkrementuj tylko, gdy sel+1 < liczby plikow
                            let files_cnt: u8; let sel: u8;
                            unsafe {
                                asm!("mov {0}, byte ptr [0x512]", out(reg_byte) files_cnt);
                                asm!("mov {0}, byte ptr [0x510]", out(reg_byte) sel);
                            }
                            if files_cnt > 0 {
                                let next = sel.wrapping_add(1);
                                if next < files_cnt { unsafe { asm!("inc byte ptr [0x510]"); } }
                            }
                            $crate::fs::fat16::refresh_mc_list();
                        }
                        else if scancode == 0x41 { // F7 - Utwórz plik
                            vga_draw_rect!(9, 7, 62, 10, 0x07); // szara ramka
                            vga_draw_rect!(10, 8, 60, 8, 0x01); // granatowe wnętrze
                            vga_print!(12, 9, 0x1F, b" UTWORZ PLIK ");
                            vga_print!(12, 11, 0x1F, b"Nazwa (8 znakow, A-Z/0-9):");
                            vga_print!(0, 24, 0x70, b" NAME > ");
                            unsafe {
                                asm!("mov byte ptr [0x511], 1"); // kontekst: tworzenie pliku
                                asm!("mov byte ptr [0x505], 0");  // reset dlugosci
                                asm!("mov byte ptr [0x506], 1");  // tryb input
                            }
                        }
                        else if scancode == 0x1C { // ENTER - Otwórz zaznaczony (gdy istnieją pliki)
                            let files_cnt: u8;
                            unsafe { asm!("mov {0}, byte ptr [0x512]", out(reg_byte) files_cnt); }
                            if files_cnt == 0 {
                                // Brak plików: ignoruj ENTER
                                vga_print!(2, 22, 0x0E, b"Brak plikow - uzyj F7 aby utworzyc");
                            } else {
                            let sel: u8;    
                            unsafe { 
                                asm!("mov {0}, byte ptr [0x510]", out(reg_byte) sel);
                                let dir_entry_ptr = ($crate::fs::fat16::ROOT_DIR_START as *mut $crate::fs::fat16::Fat16DirEntry).add(sel as usize);
                                // Otwórz plik w edytorze
                                $crate::editor::edit_file(&(*dir_entry_ptr).name);
                                // Powrót do MC po zamknięciu edytora
                                $crate::fs::fat16::fat16_mc();
                            }
                            }
                        }
                        else if scancode == 0x01 { // ESC - Powrót
                            unsafe { asm!("mov byte ptr [0x500], 0"); }
                            main_menu();
                        }
                    }
                }

                // F5 - Reboot (zawsze aktywny)
                if scancode == 0x3F {
                    unsafe { asm!("out 0x64, al", in("al") 0xFEu8); } // Fast reset
                }
            }
        }

        if scancode >= 0x80 { unsafe { asm!("mov byte ptr [0x501], 0"); } }
    }
}

#[unsafe(no_mangle)]
pub fn main_menu() {
        vga_clear!(0x00); // Czarny ekran
        unsafe {
            asm!("mov byte ptr [0x500], 0"); // Stan: Main Menu (używamy bezpiecznego adresu 0x500)
        }

        // vga_print_ext!(0, 0, b"Rusted\0", [0x0F, 0x0A, 0x0E, 0x0C, 0x0B, 0x05]);

        vga_write!(0, 0, b'R', 0x0F);
        vga_write!(1, 0, b'u', 0x0A);
        vga_write!(2, 0, b's', 0x0E);
        vga_write!(3, 0, b't', 0x0C);
        vga_write!(4, 0, b'e', 0x0B);
        vga_write!(5, 0, b'd', 0x05);

        // Click Key 1 - Mode1
        vga_print!(0, 2, 0x0F, b"Click");
        vga_print!(6, 2, 0x0C, b"Key 1");
        vga_write!(12, 2, b'-', 0x0F);
        vga_print!(14, 2, 0x0A, b"Mode1");

        // Click Key 2 - Mode2
        vga_print!(0, 3, 0x0F, b"Click");
        vga_print!(6, 3, 0x0A, b"Key 2");
        vga_write!(12, 3, b'-', 0x0F);
        vga_print!(14, 3, 0x0B, b"Mode2");

        // Click Key 3 - Mode3
        vga_print!(0, 4, 0x0F, b"Click");
        vga_print!(6, 4, 0x0E, b"Key 3");
        vga_write!(12, 4, b'-', 0x0F);
        vga_print!(14, 4, 0x0C, b"Mode3");

        // Click Key 4 - Line
        vga_print!(0, 5, 0x0F, b"Click");
        vga_print!(6, 5, 0x09, b"Key 4");
        vga_write!(12, 5, b'-', 0x0F);
        vga_print!(14, 5, 0x0D, b"Line");


        // Click Key 9 - Secrets
        vga_print!(0, 6, 0x0F, b"Click");
        vga_print!(6, 6, 0x01, b"Key 9");
        vga_write!(12, 6, b'-', 0x0F);
        vga_print!(14, 6, 0x07, b"Secrets");

        // Click Key M - Math
        vga_print!(0, 7, 0x0F, b"Click");
        vga_print!(6, 7, 0x0D, b"Key M");
        vga_write!(12, 7, b'-', 0x0F);
        vga_print!(14, 7, 0x09, b"Math");

        // Click Key 0 - Start
        vga_print!(0, 8, 0x0F, b"Click");
        vga_print!(6, 8, 0x0D, b"Key 0");
        vga_write!(12, 8, b'-', 0x0F);
        vga_print!(14, 8, 0x0E, b"Start");

        // Click ESC - Clear
        vga_print!(0, 10, 0x0F, b"Click");
        vga_print!(6, 10, 0x05, b"ESC");
        vga_write!(10, 10, b'-', 0x0F);
        vga_print!(12, 10, 0x04, b"CLEAR");

        // Click F5 - Reboot
        vga_print!(0, 11, 0x0F, b"Click");
        vga_print!(6, 11, 0x02, b"F5");
        vga_write!(10, 11, b'-', 0x0F);
        vga_print!(12, 11, 0x06, b"REBOOT");
}