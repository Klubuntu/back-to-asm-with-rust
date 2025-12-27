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


// VGA Macros

#[macro_export]
macro_rules! vga_clear {
    ($color:expr) => {
        let fill_value = (($color as u16) << 8) | (0x20u16); // 0x20 to spacja
        unsafe {
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
                let mut c = delay_per_char;
                if c > 0 {
                    asm!(
                        "2:",
                        "nop",
                        "dec {count}",
                        "jnz 2b",
                        count = inout(reg) c => _,
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
        let offset = (($row * 80 + $col) * 2) as u64;
        let value = (($color as u16) << 8) | ($char as u16);
        
        unsafe {
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

#[macro_export]
// print via LODSB instruction
macro_rules! vga_print {
    ($col:expr, $row:expr, $color:expr, $text:expr) => {
        unsafe {
            let vga_start = 0xb8000 + (($row * 80 + $col) * 2) as u64;
            let bytes = $text;
            let ptr = bytes.as_ptr();
            let len = bytes.len();
            let color = ($color as u16) << 8; // Przesuń kolor do górnego bajtu

            asm!(
                "test {len}, {len}",  // Sprawdź czy długość > 0
                "jz 3f",
                "2:",
                "movzx rax, byte ptr [rsi]", // Pobierz znak z adresu w RSI (zero-extend)
                "or rax, {clr}",      // Dodaj kolor (już przesunięty)
                "mov [rdi], ax",      // Zapisz AX (AL=znak, AH=kolor) do VGA
                "add rdi, 2",         // Następna pozycja VGA
                "add rsi, 1",         // Następny znak w pamięci
                "dec {len}",          // Zmniejsz licznik
                "jnz 2b",             // Kontynuuj jeśli len > 0
                "3:",
                clr = in(reg) color,
                len = inout(reg) len => _,
                in("rsi") ptr,        // Przekaż wskaźnik do tekstu
                in("rdi") vga_start,  // Przekaż adres VGA
                out("rax") _,         // Informujemy o użyciu RAX
                options(nostack)
            );
        }
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
macro_rules! write_char_macro {
    ($col:expr, $row:expr, $ch:expr, $color:expr) => {
        let off = (($row * 80 + $col) * 2) as u32;
        let val = (($color as u16) << 8) | ($ch as u16);
        unsafe {
            asm!(
                "mov word ptr [0xb8000 + {off:e}], {val:x}",
                off = in(reg) off,
                val = in(reg) val,
                options(nostack) // Kluczowe: informujemy, że nie ruszamy stosu
            );
        }
    };
}

#[macro_export]
macro_rules! poll_keyboard_unified {
    ($row:expr) => {
        let scancode: u8;
        unsafe {
            asm!("in al, 0x60", out("al") scancode);
        }

        // Odczyt poprzedniego stanu (używamy bezpiecznego adresu 0x501)
        let last: u8;
        unsafe {
            asm!("mov {0}, byte ptr [0x501]", out(reg_byte) last);
        }

        // Odczyt aktualnego menu (0 = Main, 1 = Math) (adres 0x500)
        let menu_state: u8;
        unsafe {
            asm!("mov {0}, byte ptr [0x500]", out(reg_byte) menu_state);
        }

        if scancode != last {
            // Zapisujemy nowy stan
            unsafe {
                asm!("mov byte ptr [0x501], {0}", in(reg_byte) scancode);
            }

            if scancode < 0x80 { // Tylko naciśnięcia
                if menu_state == 0 {
                    // --- LOGIKA MAIN MENU ---
                    if scancode == 0x02 { // Klawisz 1
                        let c = 0x0A; // Zielony
                        vga_print!(0, $row, c, b"Rusted");
                        write_char_macro!(0, $row, b'M', 0x0A);
                        write_char_macro!(1, $row, b'1', 0x0A);
                    } 
                    else if scancode == 0x03 { // Klawisz 2
                        let c = 0x0E; // Żółty
                        vga_print!(0, $row, c, b"Rusted");
                        write_char_macro!(0, $row, b'M', 0x0B);
                        write_char_macro!(1, $row, b'2', 0x0B);
                    }
                    else if scancode == 0x04 { // Klawisz 3
                        let c = 0x0C; // Czerwony
                        vga_print!(0, $row, c, b"Rusted");
                        write_char_macro!(0, $row, b'M', 0x0C);
                        write_char_macro!(1, $row, b'3', 0x0C);
                    }
                    else if scancode == 0x0A { // Klawisz 9
                        // Blokada spamu podczas animacji (używamy bajtu 0x503)
                        let anim_lock: u8;
                        unsafe { asm!("mov {0}, byte ptr [0x503]", out(reg_byte) anim_lock); }
                        if anim_lock == 0 {
                            unsafe { asm!("mov byte ptr [0x503], 1"); }

                            // Welcome on board!
                            vga_write!(0, 0, b'W', 0x04);
                            sleep_time!(500);
                            vga_write!(1, 0, b'e', 0x08);
                            sleep_time!(500);
                            vga_write!(2, 0, b'l', 0x0F);
                            sleep_time!(500);
                            vga_write!(3, 0, b'c', 0x0B);
                            sleep_time!(500);
                            vga_write!(4, 0, b'o', 0x0A);
                            sleep_time!(500);
                            vga_write!(5, 0, b'm', 0x02);
                            sleep_time!(500);
                            vga_write!(6, 0, b'e', 0x06);
                            sleep_time!(400);
                            vga_clear_animated!(0, 0, 0x00, 600, 30);
                            vga_write!(0, 0, b'o', 0x0F);
                            sleep_time!(500);
                            vga_write!(1, 0, b'n', 0x0F);
                            sleep_time!(500);
                            vga_write!(3, 0, b'B', 0x0F);
                            sleep_time!(500);
                            vga_write!(4, 0, b'0', 0x0F);
                            sleep_time!(500);
                            vga_write!(5, 0, b'a', 0x0F);
                            sleep_time!(500);
                            vga_write!(6, 0, b'R', 0x0F);
                            sleep_time!(500);
                            vga_write!(7, 0, b'd', 0x0F);
                            sleep_time!(300);
                            vga_clear_animated!(0, 0, 0x00, 600, 10);
                            vga_print!(0, 0, 0x0F, b"by Klubuntu (github.com/klubuntu)");

                            unsafe { asm!("mov byte ptr [0x503], 0"); }
                        }
                    }
                    else if scancode == 0x32 { // Klawisz M
                        unsafe { asm!("mov byte ptr [0x500], 1"); } // Zmiana stanu na Math
                        vga_clear!(0x00); // Czarny ekran
                        vga_print!(0, 0, 0x0F, b"Key 1 - Addition");
                        vga_print!(0, 1, 0x0F, b"Key 2 - Subtraction");
                        vga_print!(0, 2, 0x0F, b"Key 3 - Multiplication");
                        vga_print!(0, 3, 0x0F, b"Key 4 - Division");
                        vga_print!(0, 4, 0x0F, b"Key 9 - Random Number");
                        vga_print!(0, 5, 0x0F, b"Key 0 - Back to Main Menu");
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
                }

                if scancode == 0x3F { // F5 - Reboot (zawsze działa)
                    // 1. Reset PCI (Najskuteczniejszy w QEMU)
                    unsafe {
                        asm!(
                            "out dx, al",
                            in("dx") 0xCF9u16,
                            in("al") 0x06u8,
                            options(nomem, nostack)
                        );
                    }

                    // 2. Reset przez kontroler klawiatury (to co masz w reboot_system)
                    unsafe {
                        asm!(
                            "out 0x64, al",
                            in("al") 0xFEu8,
                            options(nomem, nostack)
                        );
                    }

                    // 3. Triple Fault (Gwarantuje, że QEMU zareaguje)
                    unsafe {
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
            unsafe {
                asm!("mov byte ptr [0x501], 0");
            }
        }
    };
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


        // Click Key 9 - Secrets
        vga_print!(0, 5, 0x0F, b"Click");
        vga_print!(6, 5, 0x01, b"Key 9");
        vga_write!(12, 5, b'-', 0x0F);
        vga_print!(14, 5, 0x07, b"Secrets");

        // Click Key M - Math
        vga_print!(0, 6, 0x0F, b"Click");
        vga_print!(6, 6, 0x0D, b"Key M");
        vga_write!(12, 6, b'-', 0x0F);
        vga_print!(14, 6, 0x09, b"Math");

        // Click Key 0 - Start
        vga_print!(0, 7, 0x0F, b"Click");
        vga_print!(6, 7, 0x0D, b"Key 0");
        vga_write!(12, 7, b'-', 0x0F);
        vga_print!(14, 7, 0x0E, b"Start");

        // Click ESC - Clear
        vga_print!(0, 9, 0x0F, b"Click");
        vga_print!(6, 9, 0x05, b"ESC");
        vga_write!(10, 9, b'-', 0x0F);
        vga_print!(12, 9, 0x04, b"CLEAR");


        // Click F5 - Reboot
        vga_print!(0, 10, 0x0F, b"Click");
        vga_print!(6, 10, 0x02, b"F5");
        vga_write!(10, 10, b'-', 0x0F);
        vga_print!(12, 10, 0x06, b"REBOOT");
}