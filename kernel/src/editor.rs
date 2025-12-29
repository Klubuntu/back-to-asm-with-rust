use crate::vga;
use crate::fs::fat16;
use core::cell::UnsafeCell;
use core::arch::asm;

const EDITOR_WIDTH: usize = 80;
const EDITOR_HEIGHT: usize = 22; 
const CONTENT_COL_OFFSET: usize = 5;
const CONTENT_WIDTH: usize = EDITOR_WIDTH - CONTENT_COL_OFFSET;
const MAX_FILE_SIZE: usize = 8192; 

pub struct TextEditor {
    content: [u8; MAX_FILE_SIZE],
    size: usize,
    cursor_x: usize,
    cursor_y: usize,
    offset: usize,  
    filename: [u8; 8], // Zmienione na 8 bajtów (standard FAT16 base)
    filename_len: usize,
    modified: bool,
}

struct EditorCell(UnsafeCell<TextEditor>);
unsafe impl Sync for EditorCell {}

static EDITOR: EditorCell = EditorCell(UnsafeCell::new(TextEditor {
    content: [0u8; MAX_FILE_SIZE],
    size: 0,
    cursor_x: 0,
    cursor_y: 0,
    offset: 0,
    filename: [b' '; 8],
    filename_len: 0,
    modified: false,
}));

#[unsafe(no_mangle)]
pub fn edit_file(filename: &[u8; 8]) {
    let editor = unsafe { &mut *EDITOR.0.get() };
    editor.load_file(filename);
    editor.run();
}

impl TextEditor {
    fn new() -> Self {
        TextEditor {
            content: [0u8; MAX_FILE_SIZE],
            size: 0,
            cursor_x: 0,
            cursor_y: 0,
            offset: 0,
            filename: [0u8; 8],
            filename_len: 0,
            modified: false,
        }
    }

    fn load_file(&mut self, filename: &[u8; 8]) {
        self.size = 0;
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.modified = false;

        // 1. Kopiowanie nazwy pliku
        for i in 0..8 {
            self.filename[i] = filename[i];
        }

        // 2. Ładowanie zawartości z FAT16
        let mut dir_entry_ptr = fat16::ROOT_DIR_START as *mut fat16::Fat16DirEntry;
        unsafe {
            for _ in 0..512 {
                if (*dir_entry_ptr).name == self.filename {
                    let cluster = (*dir_entry_ptr).first_cluster;
                    let size = (*dir_entry_ptr).file_size as usize;
                    let src_addr = fat16::DATA_REGION_START + ((cluster as u32 - 2) * 512);
                    
                    self.size = if size > MAX_FILE_SIZE { MAX_FILE_SIZE } else { size };
                    
                    // Kopiowanie bajt po bajcie do bufora edytora
                    for i in 0..self.size {
                        self.content[i] = *(src_addr as *const u8).add(i);
                    }
                    break;
                }
                dir_entry_ptr = dir_entry_ptr.add(1);
            }
        }
    }

    fn draw_editor(&self) {
        vga_clear!(0x1E); // Niebieskie tło
        
        // Nagłówek
        vga_print!(0, 0, 0x70, b" Edytor Tekstu v0.1 - [F10] Zapisz  [ESC] Wyjdz ");

        // Rysowanie treści
        for row in 0..EDITOR_HEIGHT {
            let actual_row = row + 2;
            if actual_row >= 25 { break; }

            // Numer linii
            let line_num = row + self.offset + 1;
            let mut num_buf = [b' '; 4];
            num_buf[0] = b'0' + ((line_num / 100) % 10) as u8;
            num_buf[1] = b'0' + ((line_num / 10) % 10) as u8;
            num_buf[2] = b'0' + (line_num % 10) as u8;
            vga_print!(0, actual_row as u64, 0x17, &num_buf);
            vga_write!(4, actual_row as u64, b'|', 0x1F);

            // Zawartość linii
            for col in 0..CONTENT_WIDTH {
                let idx = (row + self.offset) * CONTENT_WIDTH + col;
                if idx < self.size {
                    vga_write!((col + CONTENT_COL_OFFSET) as u64, actual_row as u64, self.content[idx], 0x1F);
                }
                }
            }
        }

    fn insert_char(&mut self, c: u8) {
        if self.size >= MAX_FILE_SIZE { return; }
        let pos = (self.cursor_y + self.offset) * CONTENT_WIDTH + self.cursor_x;
        
        if pos <= self.size {
            // Przesuwanie zawartości w prawo
            for i in (pos..self.size).rev() {
                if i + 1 < MAX_FILE_SIZE {
                    self.content[i + 1] = self.content[i];
                }
            }
            self.content[pos] = c;
            self.size += 1;
            self.modified = true;
            self.move_cursor_right();
        }
    }

    fn delete_char(&mut self) {
        let delete_pos = self.offset + self.cursor_y * CONTENT_WIDTH + self.cursor_x;

        if delete_pos >= self.size {
            return;
        }

        // Shift content left
        for i in delete_pos..(self.size - 1) {
            self.content[i] = self.content[i + 1];
        }

        self.size -= 1;
        self.modified = true;
    }

    fn backspace(&mut self) {
        let pos = (self.cursor_y + self.offset) * CONTENT_WIDTH + self.cursor_x;
        if pos > 0 && self.size > 0 {
            for i in (pos - 1)..(self.size - 1) {
                self.content[i] = self.content[i + 1];
            }
            self.size -= 1;
            self.modified = true;
            self.move_cursor_left();
        }
    }

    fn move_cursor_right(&mut self) {
        self.cursor_x += 1;
        if self.cursor_x >= CONTENT_WIDTH {
            self.cursor_x = 0;
            self.cursor_y += 1;
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = CONTENT_WIDTH - 1;
        }
    }

    fn run(&mut self) {
        loop {
            self.draw_editor();
            // Ustawienie kursora sprzętowego VGA
            let screen_x = self.cursor_x + CONTENT_COL_OFFSET;
            let screen_y = self.cursor_y + 2;
            vga::set_cursor(screen_x as u16, screen_y as u16);

            let scancode = self.read_scancode();
            if scancode == 0x01 { break; } // ESC
            if scancode == 0x44 { self.save_file(); } // F10
            
            self.handle_input(scancode);
        }
    }

    fn read_scancode(&self) -> u8 {
        let mut scancode: u8;
        unsafe {
            loop {
                let status: u8;
                asm!("in al, 0x64", out("al") status);
                if status & 0x01 != 0 {
                    asm!("in al, 0x60", out("al") scancode);
                    if scancode < 0x80 { return scancode; }
                }
            }
        }
    }

    fn handle_input(&mut self, scancode: u8) {
        match scancode {
            0x48 => if self.cursor_y > 0 { self.cursor_y -= 1 }, // Góra
            0x50 => self.cursor_y += 1,                          // Dół
            0x4B => self.move_cursor_left(),                     // Lewo
            0x4D => self.move_cursor_right(),                    // Prawo
            0x0E => self.backspace(),                            // Backspace
            0x39 => self.insert_char(b' '),                      // Space
            0x1C => self.insert_char(b'\n'),                     // Enter
            _ => {
                // Mapowanie prostych znaków (QWERTY)
                let c = match scancode {
                    0x10..=0x19 => b"qwertyuiop"[(scancode - 0x10) as usize],
                    0x1E..=0x26 => b"asdfghjkl"[(scancode - 0x1E) as usize],
                    0x2C..=0x32 => b"zxcvbnm"[(scancode - 0x2C) as usize],
                    0x33 => b',',
                    0x34 => b'.',
                    _ => 0,
                };
                if c != 0 { self.insert_char(c); }
            }
        }
    }

    fn save_file(&mut self) {
        let data = &self.content[..self.size];
        fat16::fat16_save(&self.filename, data);
        self.modified = false;
    }
}
