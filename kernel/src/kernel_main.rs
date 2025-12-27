use core::arch::asm;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    unsafe {
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

        // Click Key 9 - Secrets
        vga_write!(0, 4, b'C', 0x0F);
        vga_write!(1, 4, b'l', 0x0F);
        vga_write!(2, 4, b'i', 0x0F);
        vga_write!(3, 4, b'c', 0x0F);
        vga_write!(4, 4, b'k', 0x0F);
        vga_write!(6, 4, b'K', 0x01);
        vga_write!(7, 4, b'e', 0x01);
        vga_write!(8, 4, b'y', 0x01);
        vga_write!(10, 4, b'9', 0x01);
        vga_write!(12, 4, b'-', 0x0F);
        vga_write!(14, 4, b'S', 0x07);
        vga_write!(15, 4, b'3', 0x07);
        vga_write!(16, 4, b'c', 0x07);
        vga_write!(17, 4, b'r', 0x07);
        vga_write!(18, 4, b'E', 0x07);
        vga_write!(19, 4, b't', 0x07);
        vga_write!(20, 4, b's', 0x07);

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

        // Click ESC - Clear
        vga_write!(0, 7, b'C', 0x0F);
        vga_write!(1, 7, b'l', 0x0F);
        vga_write!(2, 7, b'i', 0x0F);
        vga_write!(3, 7, b'c', 0x0F);
        vga_write!(4, 7, b'k', 0x0F);
        vga_write!(6, 7, b'E', 0x05);
        vga_write!(7, 7, b'S', 0x05);
        vga_write!(8, 7, b'C', 0x05);
        vga_write!(10, 7, b'-', 0x0F);
        vga_write!(12, 7, b'C', 0x04);
        vga_write!(13, 7, b'L', 0x04);
        vga_write!(14, 7, b'E', 0x04);
        vga_write!(15, 7, b'A', 0x04);
        vga_write!(16, 7, b'R', 0x04);


        // Click F5 - Reboot
        vga_write!(0, 8, b'C', 0x0F);
        vga_write!(1, 8, b'l', 0x0F);
        vga_write!(2, 8, b'i', 0x0F);
        vga_write!(3, 8, b'c', 0x0F);
        vga_write!(4, 8, b'k', 0x0F);
        vga_write!(6, 8, b'F', 0x02);
        vga_write!(7, 8, b'5', 0x02);
        vga_write!(10, 8, b'-', 0x0F);
        vga_write!(12, 8, b'R', 0x06);
        vga_write!(13, 8, b'E', 0x06);
        vga_write!(14, 8, b'B', 0x06);
        vga_write!(15, 8, b'O', 0x06);
        vga_write!(16, 8, b'O', 0x06);
        vga_write!(17, 8, b'T', 0x06);

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