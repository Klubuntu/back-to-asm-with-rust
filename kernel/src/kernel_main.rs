#[warn(unused_imports)]
use core::arch::asm;
use crate::macros::main_menu;
use crate::vga::unicode_menu;
// use crate::vga::vga_clear_hd;

static mut SEED: u64 = 12345;

// Funkcja odczytująca licznik cykli procesora (TSC)
pub fn rdtsc() -> u64 {
    let rax: u64;
    let rdx: u64;
    unsafe {
        asm!("rdtsc", out("rax") rax, out("rdx") rdx);
    }
    (rdx << 32) | rax
}

// Inicjalizacja ziarna
pub fn seed_rng() {
    unsafe {
        SEED = rdtsc();
    }
}

// Implementacja LCG (używana przez systemy POSIX)
pub fn next_rand() -> u64 {
    unsafe {
        // Parametry używane w 64-bitowych generatorach
        SEED = SEED.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        SEED
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    seed_rng();
    unsafe {
        
        asm!("mov byte ptr [0x500], 0"); // Inicjalizacja menu: Main Menu
        asm!("mov byte ptr [0x509], 0"); // Flaga 720p: 0=wyłączony
        main_menu();

        asm!("mov byte ptr [0x501], 0"); // Ostatni scancode
        asm!("mov byte ptr [0x502], 0"); // Rozmiar początkowy 0
        asm!("mov byte ptr [0x503], 0"); // Flaga blokady animacji (key 9)

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