#[warn(unused_imports)]
use core::arch::asm;
use crate::vga::{Color, ColorCode};

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    unsafe {
        vga_clear!(0x00); // Czarny ekran

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

        // Click Key 0 - Start
        vga_print!(0, 6, 0x0F, b"Click");
        vga_print!(6, 6, 0x0D, b"Key 0");
        vga_write!(12, 6, b'-', 0x0F);
        vga_print!(14, 6, 0x0E, b"Start");

        // Click ESC - Clear
        vga_print!(0, 8, 0x0F, b"Click");
        vga_print!(6, 8, 0x05, b"ESC");
        vga_write!(10, 8, b'-', 0x0F);
        vga_print!(12, 8, 0x04, b"CLEAR");


        // Click F5 - Reboot
        vga_print!(0, 9, 0x0F, b"Click");
        vga_print!(6, 9, 0x02, b"F5");
        vga_write!(10, 9, b'-', 0x0F);
        vga_print!(12, 9, 0x06, b"REBOOT");

        asm!("mov byte ptr [0xb8f9e], 0"); 
        asm!("mov byte ptr [0xb8f9c], 0"); // Rozmiar początkowy 0

        loop { 
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