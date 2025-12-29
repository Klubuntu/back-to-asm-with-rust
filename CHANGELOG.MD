### 📝 Changes


#### 2025-12-29 [22:00]
* **Fix: Editor stability and bounds**
    - Naprawiono zapisy do pamięci VGA powodujące crash przy otwieraniu edytora — `draw_editor()` oraz wszystkie makra zapisu VGA zostały przejrzane i ograniczone do obszaru 80x25.
    - Wprowadzono `CONTENT_COL_OFFSET` i `CONTENT_WIDTH` w `kernel/src/editor.rs`, dzięki czemu rysowanie treści i numerów linii nie wychodzi poza granice ekranu.
    - Poprawiono logikę kursora, zawijanie i przesuwanie (`move_cursor_left`, `move_cursor_right`) oraz zabezpieczono `insert_char`, `delete_char`, `backspace` przed indeksacją poza buforem.

* **Feature: File I/O integration (FAT16 <-> Editor)**
    - `edit_file()` zmienione tak, by przyjmować nazwę pliku jako `&[u8; 8]` i `load_file()` teraz ładuje zawartość pliku z RAMDISKu FAT16 do bufora edytora.
    - Zmieniono `save_file()` w edytorze, aby używać istniejącej funkcji `fat16::fat16_save()` i zapisywać aktualny bufor jako plik w RAMDISK.

* **Safety: VGA API & macros**
    - `vga::write_char()` zyskało sprawdzenie granic (80x25) — niepoprawne zapisy są ignorowane zamiast nadpisywać pamięć.
    - Makro `write_char_macro!` zostało zastąpione wywołaniem `vga::write_char(...)` aby centralnie kontrolować walidację zapisu.
    - Dodano `vga::set_cursor()` do sterowania kursorem tekstowym (z walidacją współrzędnych).

* **Misc: Build script and helpers**
    - `make_kernel.sh` zaktualizowany, aby wymusić `rustup` nightly + `rust-src`, wykonać `cargo clean` przed budową i poprawnie wygenerować `release/kernel.bin` (zapobiegać starym binarkom podczas testów edytora).

#### 2025-12-29 [20:00]
* **Feature: FAT16 RAMDISK support**
    - Dodano wsparcie dla FAT16 w ramdisk (ładowanie i zapisy plików w pamięci RAM).

#### 2025-12-28 [16:30]
* **Bugfix: Unsafe Operation Warnings (E0133)**
    - Wrapped inline `asm!()` macros and raw pointer dereferences in explicit `unsafe` blocks within `load_polish_fonts()` in `vga.rs`.
    - Fixed Rust 2024 compatibility warnings for unsafe operations inside unsafe functions.
* **Bugfix: Duplicate Build Target Warning**
    - Changed `Cargo.toml` from `[lib]` to `[[bin]]` configuration, eliminating duplicate target warning for `src/main.rs`.
    - Single binary target now builds cleanly without conflicts.
* **Optimization: Bootloader Sector Loading**
    - Increased kernel sector loading from 70 to 140 sectors (~71.5KB) in `boot.asm` to accommodate full Rust kernel binary.
    - Kernel no longer truncates on load; full ~70KB binary is properly transferred to 0x8000.
* **Build Status**:
    - Cargo builds with zero warnings.
    - Boot sector and kernel compile without errors.

#### 2025-12-28 [15:50]
* **Refactor: VBE Graphics Mode Handling**
    - Commented out automatic VBE 1280x720 mode initialization in `boot.asm` to prevent automatic graphics mode on boot.
    - Mode `0x411B` (1280x720, 24/32-bit with LFB) now requires explicit manual triggering instead of auto-initialization.
* **Optimization**: 
    - Removed complex graphics mode implementation to maintain system stability and text mode reliability.
    - Kept text-based menu system as primary interface.
* **F8 Key**: 
    - Reserved for future graphics mode implementation (currently displays placeholder message).

#### 2025-12-28 [14:58]
* **Bugfix: Resolved Linker/Scope Error E0425**
    - Zdefiniowano brakującą funkcję `get_font_ptr`.
    - Dodano pierwsze bitmapy glifów (A, B, Crab) do statycznej pamięci kernela.
* **Refactor**: 
    - Zmieniono `draw_glyph`, aby poprawnie iterowała po 16 rzędach bitmapy, zapewniając wsparcie dla standardowych wymiarów fontu konsolowego.
* **Feature: Software Font Renderer**
    - Zaimplementowano funkcję `draw_glyph` do rysowania bitmap 8x16 w trybie LFB.
    - Dodano mapowanie bitowe (bit-shifting) dla glifów, umożliwiające renderowanie znaków piksel po pikselu.
* **Graphics**:
    - Wprowadzono obsługę kolorów 24-bit RGB w trybie graficznym.
    - Przygotowano bazę pod customowe fonty i emoji.

#### 2025-12-28 [14:50]
* **Strategy: UTF-8 & Graphics Roadmap**
    - Rozważono przejście na tryb graficzny (Framebuffer) w celu obsługi emoji.
    - Zdefiniowano potrzebę stworzenia Software Renderera dla glifów o zmiennej szerokości.
    - Zarezerwowano adres `0x508` na flagę aktywnego kodowania znaków.

#### 2025-12-28 [14:35]
* **Feature: Modifier Key Support**
    - Dodano obsługę kodu skanowania dla lewego klawisza CTRL (0x1D).
    - Wprowadzono system śledzenia stanu klawiszy modyfikujących pod adresem `0x507`.
* **Logic Update**: 
    - Przygotowano architekturę pod obsługę kombinacji klawiszy (Combo-keys).
* **Feature: Buffer-to-VGA Output**
    - Zaimplementowano odczyt bufora RAM (adres 0x600) po zatwierdzeniu klawiszem Enter.
    - Dodano dynamiczne wyświetlanie wpisanej komendy na pozycji (0, 22) przed resetem stanu.
* **Logic Fix**: 
    - Rozwiązano problem "znikających danych" poprzez wprowadzenie pętli iterującej po fizycznych adresach pamięci przed wyczyszczeniem licznika długości (0x505).

#### 2025-12-28 [14:30]
* **Architecture: Multi-level State Machine**
    - Wprowadzono adres `0x506` jako flagę `INPUT_MODE`.
    - Zaimplementowano izolację klawiatury: klawisze alfanumeryczne są procesowane tylko przy aktywnym `INPUT_MODE`.
* **UI/UX Enhancement**: 
    - Dodano obsługę klawisza ESC (0x01) jako wyjście z trybu wpisywania bez zapisu.
    - Dodano obsługę Backspace (0x0E) z dynamiczną aktualizacją bufora RAM i pamięci VGA.
* **Memory Map**:
    - `0x500`: Menu State
    - `0x505`: Input Length
    - `0x506`: Input Mode Toggle (0/1)
* **Knowledge: Keyboard Mapping Expansion**
    - Dodano obsługę kodu skanowania dla klawisza 'I' (0x17).
* **Development Status**: 
    - Rozszerzono tablicę mapowania ASCII o kolejne litery alfabetu, przygotowując system pod obsługę pełnych komend tekstowych.

#### 2025-12-28 [01:00]
* **Code Optimizations**
* **Build: Multiboot size flags**
    - W `make_multiboot.sh` dodano `--gc-sections`, strip w `objcopy` oraz optymalizacje rozmiaru w `RUSTFLAGS`, usuwając konflikt `lto`/`embed-bitcode`.
    - Dodano ścieżkę budowy kernela w trybie Multiboot (elf/bin w `release/multiboot/`).
* **UI: Migający prostokąt (Key 4)**
    - Animacja trzech kolorów (niebieski/czerwony/zielony) z opóźnieniem ~800ms zachowana, uproszczona bez tablic i pętli dla mniejszego kodu.
* **Binary size**
    - Zmniejszono narzut kodu generowanego przez pętle/zmienne w animacji, co ułatwia utrzymanie binarki ~25 KB.

#### 2025-12-28 [00:30]
* **Build: GRUB ISO script**
    - Dodano skrypt `make_iso.sh` generujący obraz ISO z GRUB i kernel Multiboot (pliki w `release/multiboot/isodir` oraz `release/rusted.iso`).

#### 2025-12-27 [23:25]
* **Feature: Unified Keyboard Polling & State Management**
    - Zaimplementowano makro `poll_keyboard_unified!`, które obsługuje logikę klawiatury w zależności od stanu menu.
    - Wprowadzono system stanów (State Machine) przechowywany pod statycznym adresem `0x500` (0: Main Menu, 1: Math Menu).
    - Dodano mechanizm "last scancode" pod adresem `0x501`, eliminujący powtarzanie akcji przy trzymaniu klawisza.
* **Feature: Interactive Math Menu**
    - Dodano dedykowany tryb obliczeń matematycznych (`menu_state == 1`).
    - Zaimplementowano operacje: dodawanie, odejmowanie, mnożenie i dzielenie z dynamicznym wyświetlaniem wyników ASCII.
    - Zintegrowano generator liczb losowych (`get_random!`) z formatowaniem dziesiętnym (obsługa dziesiątek i jedności).
* **Feature: Secrets & Animations**
    - Dodano sekwencję powitalną "Welcome on Board" pod klawiszem `9` w Main Menu.
    - Wprowadzono blokadę animacji (`anim_lock`) pod adresem `0x503`, zapobiegającą nakładaniu się procesów graficznych.
    - Dodano sygnaturę autora: `by Klubuntu (github.com/klubuntu)`.

#### 2025-12-27 [23:05]
* **Feature: PRNG (Pseudo-Random Number Generator)**
    - Zaimplementowano generator LCG (Linear Congruential Generator) operujący na 64-bitowym ziarnie.
    - Dodano obsługę instrukcji `rdtsc` przez ASM do inicjalizacji ziarna (seeding).
    - Stworzono makro `get_random!`, udostępniające losowe liczby `u64` w całym kernelu.
* **Safety**: Zastosowano operacje `wrapping`, aby uniknąć błędów przepełnienia (integer overflow) podczas generowania liczb.

#### 2025-12-27 [21:40]
* **Fix: VMA Address & Code Model**
    - Zmieniono `code-model` w `target.json` z `kernel` na `small`.
    - Rozwiązano problem błędnego adresowania (przesunięcie o 2MB), wymuszając poprawne VMA dla sekcji `.text` na poziomie `0x8000`.
* **Optimization: Binary Cleanup**
    - Dodano sekcję `/DISCARD/` w `link.ld` dla `.debug*`, `.comment` i `.eh_frame`.
    - Zastosowano flagi `-j .text -j .rodata -j .data` w `llvm-objcopy`.
    - Wyeliminowano metadane i śmieci binarne z początku pliku `.bin`, zapewniając poprawny start kernela.
* **Feature: Multi-color VGA Macro**
    - Rozbudowano makro `vga_print_ext`, wykorzystując rejestry `RSI` (tekst) oraz `RDX` (kolory).
    - Zastosowano bezpieczną pętlę sprawdzającą null-terminator przed pobraniem koloru, zapobiegając odczytom poza zakresem tablicy.
    - Wprowadzono `inout` dla rejestrów w bloku `asm!`, co wyeliminowało konflikty z LLVM.
* **Architecture: Stack Initialization & Global ASM**
    - Zastąpiono funkcję `_start` (naked) blokiem `global_asm!`.
    - Zdefiniowano bezpieczny stos o rozmiarze 16 KB w sekcji `.bss`.
    - Ustawiono `RSP` za pomocą `lea rsp, [stack_top]` oraz wyrównano stos do 16 bajtów.
    - Umożliwiono bezpieczne przekazywanie tablic tymczasowych (lokalnych) bezpośrednio w argumentach makr.

#### 2025-12-26
* **Enhancement: Nuclear Reboot Sequence**
    - Zaimplementowano wielostopniowy mechanizm restartu (F5):
        1. Port PCI `0xCF9` (Standard QEMU).
        2. Kontroler klawiatury `0x64` (Standard PS/2).
        3. Wymuszenie `Triple Fault` poprzez celowe uszkodzenie tablicy IDT (`lidt [rsp]`), gwarantujące restart w każdym środowisku.

##### 🚀 Benefits of Optimizations
* **Minimalny rozmiar binarki**: Usunięcie sekcji debugowych i komentarzy zmniejsza sporo rozmiar pliku `.bin`, co pozwala na szybsze ładowanie przez bootloader z ograniczoną liczbą sektorów.
* **Stabilność pamięci (Memory Safety)**: Dzięki rezerwacji stosu w sekcji `.bss`, mamy pewność, że dane lokalne Rusta nie nadpiszą kodu kernela ani ważnych struktur danych BIOS/UEFI.
* **Poprawne linkowanie symboli**: Model kodu `small` gwarantuje, że relatywne skoki i adresowanie danych (RIP-relative addressing) działają poprawnie w obrębie pierwszych 2GB pamięci RAM.
* **Predictable Execution**: Usunięcie śmieci z początku pliku sprawia, że pod adresem `0x8000` zawsze znajduje się instrukcja wejścia (`_start`), co eliminuje ryzyko próby wykonania przez procesor tekstowych metadanych jako kodu.
* **Feature: Advanced Animated Clear**
    - Zaktualizowano makro `vga_clear_animated!` o obsługę dowolnej liczby znaków.
    - Wprowadzono automatyczne zawijanie wierszy (word wrap) podczas czyszczenia.
    - Zoptymalizowano pętlę opóźniającą w ASM, by dopasowywała tempo animacji do liczby znaków.
* **Reduced size from 28KB to 17KB**