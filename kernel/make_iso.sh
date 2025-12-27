#!/bin/bash

# Create bootable ISO with GRUB
cd "$(dirname "$0")"

# Build kernel first
./make_multiboot.sh || exit 1

# Create directory structure for ISO in release/multiboot
mkdir -p ../release/multiboot/isodir/boot/grub

# Copy kernel
cp ../release/multiboot/kernel_multiboot.elf ../release/multiboot/isodir/boot/kernel.elf

# Create GRUB config
cat > ../release/multiboot/isodir/boot/grub/grub.cfg << 'EOF'
set timeout=0
set default=0

menuentry "Rusted OS" {
    multiboot2 /boot/kernel.elf
    boot
}
EOF

# Create ISO in main release folder
if command -v grub-mkrescue &> /dev/null; then
    grub-mkrescue -o ../release/rusted.iso ../release/multiboot/isodir/
    echo "ISO created: ../release/rusted.iso"
    echo ""
    echo "Test with: qemu-system-x86_64 -cdrom ../release/rusted.iso"
elif command -v grub2-mkrescue &> /dev/null; then
    grub2-mkrescue -o ../release/rusted.iso ../release/multiboot/isodir/
    echo "ISO created: ../release/rusted.iso"
    echo ""
    echo "Test with: qemu-system-x86_64 -cdrom ../release/rusted.iso"
else
    echo "Error: grub-mkrescue not found!"
    echo "Install with: sudo apt install grub-pc-bin xorriso"
    exit 1
fi
