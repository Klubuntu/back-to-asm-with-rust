# Multiboot / GRUB Support

## Struktura
- `multiboot.asm` - nagłówek Multiboot i 32->64 bit transition
- `link_multiboot.ld` - linker script dla Multiboot
- `make_multiboot.sh` - kompilacja kernela z Multiboot
- `make_iso.sh` - tworzenie bootable ISO z GRUB

## Kompilacja

### Kernel z Multiboot (Multiboot2)
```bash
./make_multiboot.sh
```
Tworzy: `kernel_multiboot.elf`

### Bootable ISO
```bash
./make_iso.sh
```
Tworzy: `rusted.iso`

## Wymagania
```bash
# Debian/Ubuntu
sudo apt install grub-pc-bin xorriso nasm

# Fedora
sudo dnf install grub2-tools xorriso nasm
```

## Uruchomienie

### QEMU z ISO
```bash
qemu-system-x86_64 -cdrom rusted.iso
```

### QEMU z plikiem ELF (bez ISO)
```bash
qemu-system-x86_64 -kernel kernel_multiboot.elf
```

## Różnice między trybami

### Oryginalny bootloader (boot.asm)
- Własny bootloader w MBR
- Ładuje kernel z sektora 2
- 512-bajtowy boot sector
- Uruchamia: `./connect_and_run.sh`

### Multiboot (GRUB)
- Używa GRUB jako bootloadera
- Standardowy format Multiboot
- Bootable ISO
- Uruchamia: `qemu-system-x86_64 -cdrom rusted.iso`

## Struktura nagłówka Multiboot2

```asm
; Multiboot2 header (używany przez ten projekt)
section .multiboot
align 8
multiboot2_header:
    dd 0xE85250D6                                ; Magic (Multiboot2)
    dd 0                                         ; Arch: i386
    dd multiboot2_header_end - multiboot2_header ; Header length
    dd -(0xE85250D6 + 0 + (multiboot2_header_end - multiboot2_header)) ; Checksum

    ; End tag
    dw 0
    dw 0
    dd 8
multiboot2_header_end:
```

- Magic: `0xE85250D6` — Multiboot2
- Header musi znaleźć się w pierwszych 8KB pliku i być poprawnie wyrównany
- Ten projekt używa Multiboot2, nie Multiboot1

## Debugging

Sprawdź czy ELF ma poprawny nagłówek Multiboot2 (ten projekt używa MB2):
```bash
grub-file --is-x86-multiboot2 kernel_multiboot.elf && echo "OK" || echo "FAIL"
```

Uwaga: komenda `--is-x86-multiboot` sprawdza Multiboot1. Dla naszego pliku ELF (Multiboot2) zwróci `FAIL`, co jest spodziewane.

Zobacz strukturę pliku:
```bash
readelf -h kernel_multiboot.elf
objdump -x kernel_multiboot.elf | grep multiboot
```
