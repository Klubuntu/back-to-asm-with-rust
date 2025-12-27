// Utils macros
macro_rules! sleep_time { // in ms
    ($seconds:expr) => {
        // Przyjmujemy, że ok. 2 000 000 000 pętli to mniej więcej 1 sekunda 
        // (wartość zależy od taktowania, w QEMU może być różnie)
        let iterations = $seconds as u64 * 100_000; 
        asm!(
            "2:",
            "nop",
            "dec {count}",
            "jnz 2b",
            count = inout(reg) iterations => _,
            options(nostack, preserves_flags)
        );
    };
}


// VGA Macros

#[macro_export]
macro_rules! vga_clear {
    ($color:expr) => {
        let fill_value = (($color as u16) << 8) | (0x20u16); // 0x20 to spacja
        asm!(
            "cld",           // Wyczyść flagę kierunku (zwiększanie EDI)
            "rep stosw",     // Powtarzaj 'store word' (AX -> [EDI])
            inout("rcx") 80 * 25 => _, // Licznik powtórzeń (CX/RCX)
            inout("rdi") 0xb8000 => _, // Adres docelowy (DI/RDI)
            in("ax") fill_value,       // Wartość do wpisania
            options(nostack, preserves_flags)
        );
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
        
        asm!(
            // Używamy bezpośredniego adresu 0xb8000
            // {off:e} wymusza użycie 32-bitowej części rejestru (np. eax zamiast rax)
            "mov word ptr [0xb8000 + {off:e}], {val:x}",
            off = in(reg) offset,
            val = in(reg) value,
            options(nostack, preserves_flags)
        );
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
            let color = $color as u8;

            asm!(
                "mov ah, {clr_byte}", // Załaduj kolor do AH
                "2:",
                "mov al, [rsi]",      // Pobierz znak z adresu w RSI
                "test al, al",        // Czy to null-terminator?
                "jz 3f",              // Jeśli tak, wyjdź
                "mov [rdi], ax",      // Zapisz AX (AL=znak, AH=kolor) do VGA
                "add rdi, 2",         // Następna pozycja VGA
                "add rsi, 1",         // Następny znak w pamięci
                "jmp 2b",
                "3:",
                clr_byte = in(reg_byte) color,
                in("rsi") ptr,        // Przekaż wskaźnik do tekstu
                in("rdi") vga_start,  // Przekaż adres VGA
                out("ax") _,          // Informujemy o użyciu AX
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

            asm!(
                "2:",
                "mov al, [rsi]",      // Pobierz znak
                "test al, al",        // Czy koniec?
                "jz 3f",
                "mov ah, [rdx]",      // Pobierz kolor
                "mov [rdi], ax",      // Zapisz do VGA
                "add rsi, 1",
                "add rdx, 1",
                "add rdi, 2",
                "jmp 2b",
                "3:",
                inout("rsi") text_ptr => _,   // Wejście i ignorowane wyjście
                inout("rdx") color_ptr => _,  // To rozwiązuje Twój błąd!
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
        asm!(
            "mov word ptr [0xb8000 + {off:e}], {val:x}",
            off = in(reg) off,
            val = in(reg) val,
            options(nostack) // Kluczowe: informujemy, że nie ruszamy stosu
        );
    };
}

#[macro_export]
macro_rules! poll_keyboard_unified {
    ($row:expr) => {
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

        // Reset schowka przy puszczeniu klawisza (Break Code)
        if scancode >= 0x80 {
            asm!("mov byte ptr [0xb8f9e], 0");
        }
    };
}
