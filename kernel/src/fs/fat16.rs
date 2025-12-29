// =============================================================================
// SYSTEM PLIKÓW FAT16 - RAMDISK (Bare Metal Implementation)
// =============================================================================
// Mapa pamięci:
// 0x100000 - Boot Sector (1 sektor)
// 0x100200 - FAT Table 1 (Tablica powiązań klastrów)
// 0x104200 - FAT Table 2 (Kopia zapasowa)
// 0x108200 - Root Directory (Lista plików)
// 0x10C200 - Data Region (Miejsce na treść plików)
// =============================================================================

use core::arch::asm;

// Stałe adresy pamięci RAM dla systemu plików
pub const RAMDISK_BASE: u32 = 0x100000;
pub const FAT_TABLE_START: u32 = 0x100200;
pub const ROOT_DIR_START: u32 = 0x108200;
pub const DATA_REGION_START: u32 = 0x10C200;

#[repr(C, packed)]
pub struct Fat16DirEntry {
    pub name: [u8; 8],      // Nazwa pliku (8 znaków)
    pub ext: [u8; 3],       // Rozszerzenie (3 znaki)
    pub attr: u8,           // Atrybuty (0x20 = plik archiwalny)
    pub reserved: u8,       
    pub creation_time_ms: u8,
    pub creation_time: u16,
    pub creation_date: u16,
    pub last_access_date: u16,
    pub cluster_high: u16,  // Zawsze 0 w FAT16
    pub m_time: u16,
    pub m_date: u16,
    pub first_cluster: u16, // Indeks pierwszego klastra w danych
    pub file_size: u32,     // Rozmiar w bajtach
}

// --- MAKRA ---

#[macro_export]
macro_rules! fat16_init {
    () => {
        unsafe {
            let base = 0x100000 as *mut u8;
            
            // Czyścimy obszar pod Boot Sector i tablice (uproszczenie)
            for i in 0..0x10000 {
                *base.add(i) = 0;
            }

            // 1. BIOS Parameter Block (BPB)
            *(base.add(11) as *mut u16) = 512;  // Bytes per sector
            *(base.add(13)) = 1;                // Sectors per cluster
            *(base.add(14) as *mut u16) = 1;    // Reserved sectors (Boot Sector)
            *(base.add(16)) = 2;                // Number of FATs
            *(base.add(17) as *mut u16) = 512;  // Root entries (max plików)
            *(base.add(22) as *mut u16) = 32;   // FAT size (sektory na tablicę)
            
            // 2. Inicjalizacja tablicy FAT (klastry zarezerwowane)
            // Pierwsze dwa wpisy w FAT muszą być specjalne (F8 FF FF FF)
            let fat = 0x100200 as *mut u16;
            *fat = 0xFFF8;
            *fat.add(1) = 0xFFFF;
        }
    };
}

// --- FUNKCJE ---

#[unsafe(no_mangle)]
pub fn fat16_save_input_as_file(filename_8char: &[u8; 8]) {
    unsafe {
        // 1. Odczytujemy rozmiar wpisanego tekstu z Twojego adresu 0x505
        let mut text_len: u8;
        asm!("mov {0}, byte ptr [0x505]", out(reg_byte) text_len);
        
        if text_len == 0 { return; }

        // 2. Szukamy wolnego wpisu w Root Directory (0x108200)
        let mut dir_entry_ptr = ROOT_DIR_START as *mut Fat16DirEntry;
        let mut found_slot = false;

        for _ in 0..512 {
            if (*dir_entry_ptr).name[0] == 0x00 || (*dir_entry_ptr).name[0] == 0xE5 {
                found_slot = true;
                break;
            }
            dir_entry_ptr = dir_entry_ptr.add(1);
        }

        if !found_slot { return; }

        // 3. Szukamy wolnego klastra w tablicy FAT (0x100200)
        let fat_ptr = FAT_TABLE_START as *mut u16;
        let mut free_cluster: u16 = 0;

        for i in 2..4096 { // Zaczynamy od 2, klastry 0 i 1 są zarezerwowane
            if *fat_ptr.add(i as usize) == 0x0000 {
                free_cluster = i;
                break;
            }
        }

        if free_cluster == 0 { return; }

        // 4. Kopiujemy dane z bufora 0x600 do Data Region
        // Adres = Początek_Danych + (Cluster - 2) * Bajty_na_Sektor
        let dest_addr = DATA_REGION_START + ((free_cluster as u32 - 2) * 512);
        let src_addr = 0x600 as *const u8;

        for i in 0..text_len as u32 {
            *(dest_addr as *mut u8).add(i as usize) = *src_addr.add(i as usize);
        }

        // 5. Aktualizujemy Tablicę FAT (zamykamy łańcuch klastra)
        *fat_ptr.add(free_cluster as usize) = 0xFFFF; // Koniec pliku

        // 6. Wypełniamy strukturę Directory Entry
        let entry = &mut (*dir_entry_ptr);
        for i in 0..8 { entry.name[i] = filename_8char[i]; }
        entry.ext = *b"TXT";
        entry.attr = 0x20;
        entry.first_cluster = free_cluster;
        entry.file_size = text_len as u32;

        // 7. Informujemy o sukcesie na ekranie (opcjonalnie)
        // vga_print!(0, 23, 0x0A, b"PLIK ZAPISANY W RAM");
    }
}

#[unsafe(no_mangle)]
pub fn ramfs_list_files() {
    // Funkcja iteruje po Root Directory i wypisuje nazwy plików na ekranie
    let mut dir_entry_ptr = ROOT_DIR_START as *mut Fat16DirEntry;
    let mut row = 10;

    unsafe {
        for _ in 0..10 { // Pokaż pierwsze 10 plików
            if (*dir_entry_ptr).name[0] != 0 {
                // Ręczne wypisanie nazwy (uproszczone vga_write)
                let row_offset = row * 160;
                for i in 0..8 {
                    let c = (*dir_entry_ptr).name[i];
                    asm!("mov byte ptr [0xb8000 + {row_offset} + {col}*2], {val}",
                         row_offset = in(reg) row_offset,
                         col = in(reg) i,
                         val = in(reg_byte) c);
                }
                row += 1;
            }
            dir_entry_ptr = dir_entry_ptr.add(1);
        }
    }
}

#[unsafe(no_mangle)]
pub fn fat16_save(filename_8char: &[u8; 8], data: &[u8]) {
    unsafe {
        let data_len = data.len() as u32;
        if data_len == 0 { return; }

        // 1. Szukaj wolnego slotu w Root Directory
        let mut dir_entry_ptr = ROOT_DIR_START as *mut Fat16DirEntry;
        let mut found_slot = false;
        for _ in 0..512 {
            if (*dir_entry_ptr).name[0] == 0x00 || (*dir_entry_ptr).name[0] == 0xE5 {
                found_slot = true;
                break;
            }
            dir_entry_ptr = dir_entry_ptr.add(1);
        }
        if !found_slot { return; }

        // 2. Szukaj wolnego klastra
        let fat_ptr = FAT_TABLE_START as *mut u16;
        let mut free_cluster: u16 = 0;
        for i in 2..4096 {
            if *fat_ptr.add(i as usize) == 0x0000 {
                free_cluster = i;
                break;
            }
        }
        if free_cluster == 0 { return; }

        // 3. Kopiuj dane (z dowolnego bufora 'data')
        let dest_addr = DATA_REGION_START + ((free_cluster as u32 - 2) * 512);
        for i in 0..data_len {
            *(dest_addr as *mut u8).add(i as usize) = data[i as usize];
        }

        // 4. Zamknij klaster i zapisz meta-dane
        *fat_ptr.add(free_cluster as usize) = 0xFFFF;
        let entry = &mut (*dir_entry_ptr);
        for i in 0..8 { entry.name[i] = filename_8char[i]; }
        entry.ext = *b"TXT";
        entry.attr = 0x20;
        entry.first_cluster = free_cluster;
        entry.file_size = data_len;
    }
}

#[unsafe(no_mangle)]
pub fn fat16_read(filename_8char: &[u8; 8]) {
    // Czyta plik i wypisuje go w sekcji podglądu (np. od linii 15)
    let mut dir_entry_ptr = ROOT_DIR_START as *mut Fat16DirEntry;
    unsafe {
        for _ in 0..512 {
            if (*dir_entry_ptr).name == *filename_8char {
                let cluster = (*dir_entry_ptr).first_cluster;
                let size = (*dir_entry_ptr).file_size;
                let src_addr = DATA_REGION_START + ((cluster as u32 - 2) * 512);
                
                vga_print!(0, 14, 0x0B, b"ZAWARTOSC PLIKU:");
                for i in 0..size {
                    let c = *(src_addr as *const u8).add(i as usize);
                    vga_write!(i as u64, 15, c, 0x0F);
                }
                return;
            }
            dir_entry_ptr = dir_entry_ptr.add(1);
        }
    }
}

// --- FAT16 MINI-COMMANDER (MC) ---

#[unsafe(no_mangle)]
pub fn fat16_mc() {
    vga_clear!(0x00);
    vga_draw_rect!(1, 1, 78, 22, 0x01); // Ramka tła (granatowa)
    vga_print!(2, 1, 0x1F, b" FAT16 MINI-COMMANDER ");
    vga_print!(2, 22, 0x0F, b" [UP/DOWN] Wybor  [F7] Nowy  [ESC] Wyjdz ");

    refresh_mc_list();
}

pub fn refresh_mc_list() {
    let dir_entry_ptr = ROOT_DIR_START as *mut Fat16DirEntry;
    let mut selection: u8;
    unsafe { asm!("mov {0}, byte ptr [0x510]", out(reg_byte) selection); }

    let mut found_count: u8 = 0;
    unsafe {
        for i in 0..15 { // Lista do 15 plików
            let entry = &(*dir_entry_ptr.add(i));
            if entry.name[0] != 0 {
                let color = if i as u8 == selection { 0x70 } else { 0x1F }; // Inwersja dla zaznaczenia
                
                // Rysuj nazwę pliku
                for n in 0..8 {
                    vga_write!(4 + n as u64, 4 + found_count as u64, entry.name[n], color);
                }
                vga_print!(13, 4 + found_count as u64, color, b".TXT");
                found_count += 1;
            }
        }
        // Zapisz liczbe plikow do prostego stanu (0x512) i zaktualizuj stopkę
        asm!("mov byte ptr [0x512], {0}", in(reg_byte) found_count);
        if found_count == 0 {
            vga_print!(2, 22, 0x0E, b" Brak plikow. [F7] Nowy  [ESC] Wyjdz ");
        } else {
            vga_print!(2, 22, 0x0F, b" [UP/DOWN] Wybor  [ENTER] Otworz  [F7] Nowy  [ESC] Wyjdz ");
        }
    }
}

#[unsafe(no_mangle)]
pub fn fat16_create_file(filename_8char: &[u8; 8]) {
    unsafe {
        // 1. Znajdź wolny slot w Root Directory
        let mut dir_entry_ptr = ROOT_DIR_START as *mut Fat16DirEntry;
        let mut found_slot = false;
        for _ in 0..512 {
            if (*dir_entry_ptr).name[0] == 0x00 || (*dir_entry_ptr).name[0] == 0xE5 {
                found_slot = true;
                break;
            }
            dir_entry_ptr = dir_entry_ptr.add(1);
        }
        if !found_slot { return; }

        // 2. Znajdź wolny klaster (alokujemy nawet dla pustego pliku)
        let fat_ptr = FAT_TABLE_START as *mut u16;
        let mut free_cluster: u16 = 0;
        for i in 2..4096 {
            if *fat_ptr.add(i as usize) == 0x0000 {
                free_cluster = i;
                break;
            }
        }
        if free_cluster == 0 { return; }

        // 3. Oznacz koniec łańcucha (pusty plik)
        *fat_ptr.add(free_cluster as usize) = 0xFFFF;

        // 4. Wypełnij wpis w katalogu
        let entry = &mut (*dir_entry_ptr);
        for i in 0..8 { entry.name[i] = filename_8char[i]; }
        entry.ext = *b"TXT";
        entry.attr = 0x20;
        entry.first_cluster = free_cluster;
        entry.file_size = 0;
    }
}