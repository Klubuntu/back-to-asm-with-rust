# Multiboot / GRUB Support

## Struktura
- `multiboot.asm` - nagłówek Multiboot i 32->64 bit transition
- `link_multiboot.ld` - linker script dla Multiboot
- `make_multiboot.sh` - kompilacja kernela z Multiboot
- `make_iso.sh` - tworzenie bootable ISO z GRUB

## Kompilacja

### Kernel z Multiboot
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

## Struktura nagłówka Multiboot

```asm
section .multiboot
align 4
    dd 0x1BADB002              ; Magic number
    dd 0x00000000              ; Flags
    dd -(0x1BADB002 + 0x00000000) ; Checksum
```

- **Magic number**: `0x1BADB002` - identyfikacja Multiboot
- **Flags**: `0x00000000` - brak specjalnych flag
- **Checksum**: suma magic + flags + checksum = 0

## Debugging

Sprawdź czy ELF ma poprawny nagłówek Multiboot:
```bash
grub-file --is-x86-multiboot kernel_multiboot.elf && echo "OK" || echo "FAIL"
```

Zobacz strukturę pliku:
```bash
readelf -h kernel_multiboot.elf
objdump -x kernel_multiboot.elf | grep multiboot
```
