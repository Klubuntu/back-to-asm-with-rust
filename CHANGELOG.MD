### 📝 Changes

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